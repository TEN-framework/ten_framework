//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::os::raw::c_char;
use std::ptr;
use std::{ffi::CStr, thread};

use actix_web::{web, App, HttpResponse, HttpServer};
use anyhow::Result;
use futures::channel::oneshot;
use prometheus::{
    Counter, CounterVec, Encoder, Gauge, GaugeVec, Histogram, HistogramOpts,
    HistogramVec, Opts, Registry, TextEncoder,
};

use crate::constants::{METRIC_DEFAULT_PATH, METRIC_DEFAULT_URL};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Label {
    pub key: *const c_char,
    pub value: *const c_char,
}

pub struct MetricSystem {
    registry: Registry,

    actix_thread: Option<thread::JoinHandle<()>>,

    /// Used to send a shutdown signal to the actix system where the server is
    /// located.
    actix_shutdown_tx: Option<oneshot::Sender<()>>,
}

pub enum MetricHandle {
    Counter(Counter),
    CounterVec {
        vec: CounterVec,
        label_values: Vec<String>,
    },
    Gauge(Gauge),
    GaugeVec {
        vec: GaugeVec,
        label_values: Vec<String>,
    },
    Histogram(Histogram),
    HistogramVec {
        vec: HistogramVec,
        label_values: Vec<String>,
    },
}

/// Initialize the metric system.
///
/// # Parameter
/// - `url`: The full address including port information, e.g., "127.0.0.1:9090"
/// - `path`: The HTTP endpoint path, e.g., "/metrics"
///
/// # Return value
/// Returns a pointer to `MetricSystem` on success, otherwise returns `null`.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ten_metric_system_create(
    url: *const c_char,
    path: *const c_char,
) -> *mut MetricSystem {
    let url_str = if url.is_null() {
        METRIC_DEFAULT_URL.to_string()
    } else {
        let c_str_url = unsafe { CStr::from_ptr(url) };
        match c_str_url.to_str() {
            Ok(s) if !s.trim().is_empty() => s.to_string(),
            _ => METRIC_DEFAULT_URL.to_string(),
        }
    };

    let path_str = if path.is_null() {
        METRIC_DEFAULT_PATH.to_string()
    } else {
        let c_str_path = unsafe { CStr::from_ptr(path) };
        match c_str_path.to_str() {
            Ok(s) if !s.trim().is_empty() => s.to_string(),
            _ => METRIC_DEFAULT_PATH.to_string(),
        }
    };

    // Note: `prometheus::Registry` internally uses `Arc` and `RwLock` to
    // achieve thread safety, so there is no need to add additional locking
    // mechanisms. It can be used directly here.
    let registry = Registry::new();

    // Start the actix-web server to provide metrics data at the specified path.
    let registry_clone = registry.clone();
    let path_clone = path_str.clone();

    let server_builder = HttpServer::new(move || {
        App::new().route(
            &path_clone,
            web::get().to({
                let registry_handler = registry_clone.clone();

                move || {
                    let registry_for_request = registry_handler.clone();

                    async move {
                        let metric_families = registry_for_request.gather();
                        let encoder = TextEncoder::new();
                        let mut buffer = Vec::new();

                        if encoder
                            .encode(&metric_families, &mut buffer)
                            .is_err()
                        {
                            return HttpResponse::InternalServerError()
                                .finish();
                        }

                        let response = match String::from_utf8(buffer) {
                            Ok(v) => v,
                            Err(_) => {
                                return HttpResponse::InternalServerError()
                                    .finish()
                            }
                        };

                        HttpResponse::Ok().body(response)
                    }
                }
            }),
        )
    })
    .bind(&url_str);

    let server_builder = match server_builder {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Error binding to address: {}", url_str);
            return ptr::null_mut();
        }
    };

    let server = server_builder.run();

    // Create an `oneshot` channel to notify the actix system to shut down.
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let server_thread_handle = thread::spawn(move || {
        let sys = actix_rt::System::new();

        // Register a task in the same actix system to wait for the
        // `shutdown_rx` signal. Once received, call `System::current().stop()`.
        actix_rt::spawn(async move {
            let _ = shutdown_rx.await;

            // Shut down the current actix system, causing `block_on(server)` to
            // exit.
            actix_rt::System::current().stop();
        });

        // Run the server until the system stops.
        let _ = sys.block_on(server);
    });

    let actix_thread = Some(server_thread_handle);

    let system = MetricSystem {
        registry,
        actix_thread,
        actix_shutdown_tx: Some(shutdown_tx),
    };

    Box::into_raw(Box::new(system))
}

/// Shut down the metric system, stop the server, and clean up all resources.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ten_metric_system_shutdown(system_ptr: *mut MetricSystem) {
    debug_assert!(!system_ptr.is_null(), "System pointer is null");

    if system_ptr.is_null() {
        return;
    }

    // Retrieve ownership using `Box::from_raw`, and it will be automatically
    // released when the function exits.
    let system = unsafe { Box::from_raw(system_ptr) };

    // Notify the actix system to shut down through the `oneshot` channel.
    if let Some(shutdown_tx) = system.actix_shutdown_tx {
        let _ = shutdown_tx.send(());
    }

    if let Some(server_thread_handle) = system.actix_thread {
        let _ = server_thread_handle.join();
    }
}

fn convert_labels(
    labels_ptr: *const Label,
    labels_len: usize,
) -> Option<Vec<(String, String)>> {
    if labels_ptr.is_null() {
        return Some(vec![]);
    }

    let mut result = Vec::with_capacity(labels_len);

    for i in 0..labels_len {
        let label = unsafe { *labels_ptr.add(i) };

        if label.key.is_null() || label.value.is_null() {
            return None;
        }

        let key_cstr = unsafe { CStr::from_ptr(label.key) };
        let value_cstr = unsafe { CStr::from_ptr(label.value) };

        let key = match key_cstr.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return None,
        };

        let value = match value_cstr.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return None,
        };

        result.push((key, value));
    }

    Some(result)
}

fn create_metric_counter(
    system: &mut MetricSystem,
    name_str: &str,
    help_str: &str,
) -> Result<MetricHandle> {
    let counter_opts = Opts::new(name_str, help_str);
    match Counter::with_opts(counter_opts) {
        Ok(counter) => {
            if let Err(e) = system.registry.register(Box::new(counter.clone()))
            {
                eprintln!("Error registering counter: {:?}", e);
                return Err(anyhow::anyhow!("Error registering counter"));
            }
            Ok(MetricHandle::Counter(counter))
        }
        Err(_) => Err(anyhow::anyhow!("Error creating counter")),
    }
}

fn create_metric_counter_with_labels(
    system: &mut MetricSystem,
    name_str: &str,
    help_str: &str,
    label_names: &Vec<&str>,
    label_values: &[String],
) -> Result<MetricHandle> {
    let counter_opts = Opts::new(name_str, help_str);
    match CounterVec::new(counter_opts, label_names) {
        Ok(counter_vec) => {
            if let Err(e) =
                system.registry.register(Box::new(counter_vec.clone()))
            {
                eprintln!("Error registering counter vec: {:?}", e);
                return Err(anyhow::anyhow!("Error registering counter"));
            }
            Ok(MetricHandle::CounterVec {
                vec: counter_vec,
                label_values: label_values.to_vec(),
            })
        }
        Err(_) => Err(anyhow::anyhow!("Error creating counter")),
    }
}

fn create_metric_gauge(
    system: &mut MetricSystem,
    name_str: &str,
    help_str: &str,
) -> Result<MetricHandle> {
    let gauge_opts = Opts::new(name_str, help_str);
    match Gauge::with_opts(gauge_opts) {
        Ok(gauge) => {
            if let Err(e) = system.registry.register(Box::new(gauge.clone())) {
                eprintln!("Error registering gauge: {:?}", e);
                return Err(anyhow::anyhow!("Error registering gauge"));
            }
            Ok(MetricHandle::Gauge(gauge))
        }
        Err(_) => Err(anyhow::anyhow!("Error creating gauge")),
    }
}

fn create_metric_gauge_with_labels(
    system: &mut MetricSystem,
    name_str: &str,
    help_str: &str,
    label_names: &Vec<&str>,
    label_values: &[String],
) -> Result<MetricHandle> {
    let gauge_opts = Opts::new(name_str, help_str);
    match GaugeVec::new(gauge_opts, label_names) {
        Ok(gauge_vec) => {
            if let Err(e) =
                system.registry.register(Box::new(gauge_vec.clone()))
            {
                eprintln!("Error registering gauge vec: {:?}", e);
                return Err(anyhow::anyhow!("Error registering gauge"));
            }
            Ok(MetricHandle::GaugeVec {
                vec: gauge_vec,
                label_values: label_values.to_vec(),
            })
        }
        Err(_) => Err(anyhow::anyhow!("Error creating gauge")),
    }
}

fn create_metric_histogram(
    system: &mut MetricSystem,
    name_str: &str,
    help_str: &str,
) -> Result<MetricHandle> {
    let hist_opts = HistogramOpts::new(name_str, help_str);
    match Histogram::with_opts(hist_opts) {
        Ok(histogram) => {
            if let Err(e) =
                system.registry.register(Box::new(histogram.clone()))
            {
                eprintln!("Error registering histogram: {:?}", e);
                return Err(anyhow::anyhow!("Error registering histogram"));
            }
            Ok(MetricHandle::Histogram(histogram))
        }
        Err(_) => Err(anyhow::anyhow!("Error creating histogram")),
    }
}

fn create_metric_histogram_with_labels(
    system: &mut MetricSystem,
    name_str: &str,
    help_str: &str,
    label_names: &Vec<&str>,
    label_values: &[String],
) -> Result<MetricHandle> {
    let hist_opts = HistogramOpts::new(name_str, help_str);
    match HistogramVec::new(hist_opts, label_names) {
        Ok(histogram_vec) => {
            if let Err(e) =
                system.registry.register(Box::new(histogram_vec.clone()))
            {
                eprintln!("Error registering histogram vec: {:?}", e);
                return Err(anyhow::anyhow!("Error registering histogram"));
            }
            Ok(MetricHandle::HistogramVec {
                vec: histogram_vec,
                label_values: label_values.to_vec(),
            })
        }
        Err(_) => Err(anyhow::anyhow!("Error creating histogram")),
    }
}

/// Create a metric.
///
/// # Parameter
/// - `system_ptr`: Pointer to the previously created MetricSystem.
/// - `metric_type`: 0 for Counter, 1 for Gauge, 2 for Histogram.
/// - `name`, `help`: The name and description of the metric.
/// - `labels_ptr` and `labels_len`: If not null, a metric with labels will be
///   created; it is assumed that a fixed set of labels is provided (label
///   values are determined at creation time, and do not need to be specified
///   during updates).
///
/// # Return
/// Returns a pointer to MetricHandle on success, otherwise returns null.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ten_metric_create(
    system_ptr: *mut MetricSystem,
    metric_type: u32, // 0=Counter, 1=Gauge, 2=Histogram
    name: *const c_char,
    help: *const c_char,
    labels_ptr: *const Label,
    labels_len: usize,
) -> *mut MetricHandle {
    debug_assert!(!system_ptr.is_null(), "System pointer is null");
    debug_assert!(!name.is_null(), "Name is null for metric creation");
    debug_assert!(!help.is_null(), "Help is null for metric creation");

    if system_ptr.is_null() || name.is_null() || help.is_null() {
        return ptr::null_mut();
    }

    let system = unsafe { &mut *system_ptr };

    let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let help_str = match unsafe { CStr::from_ptr(help) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let label_pairs = match convert_labels(labels_ptr, labels_len) {
        Some(v) => v,
        None => return ptr::null_mut(),
    };

    let mut label_names = Vec::new();
    let mut label_values = Vec::new();
    for (k, v) in label_pairs.iter() {
        label_names.push(k.as_str());
        label_values.push(v.clone());
    }

    let metric_handle = match metric_type {
        0 => {
            // Counter.
            if label_names.is_empty() {
                match create_metric_counter(system, name_str, help_str) {
                    Ok(metric) => metric,
                    Err(_) => return ptr::null_mut(),
                }
            } else {
                match create_metric_counter_with_labels(
                    system,
                    name_str,
                    help_str,
                    &label_names,
                    &label_values,
                ) {
                    Ok(metric) => metric,
                    Err(_) => return ptr::null_mut(),
                }
            }
        }
        1 => {
            // Gauge.
            if label_names.is_empty() {
                match create_metric_gauge(system, name_str, help_str) {
                    Ok(metric) => metric,
                    Err(_) => return ptr::null_mut(),
                }
            } else {
                match create_metric_gauge_with_labels(
                    system,
                    name_str,
                    help_str,
                    &label_names,
                    &label_values,
                ) {
                    Ok(metric) => metric,
                    Err(_) => return ptr::null_mut(),
                }
            }
        }
        2 => {
            // Histogram.
            if label_names.is_empty() {
                match create_metric_histogram(system, name_str, help_str) {
                    Ok(metric) => metric,
                    Err(_) => return ptr::null_mut(),
                }
            } else {
                match create_metric_histogram_with_labels(
                    system,
                    name_str,
                    help_str,
                    &label_names,
                    &label_values,
                ) {
                    Ok(metric) => metric,
                    Err(_) => return ptr::null_mut(),
                }
            }
        }
        _ => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(metric_handle))
}

/// Release a metric handle created by `ten_metric_create`.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ten_metric_destroy(metric_ptr: *mut MetricHandle) {
    debug_assert!(!metric_ptr.is_null(), "Metric pointer is null");

    if metric_ptr.is_null() {
        return;
    }

    unsafe {
        let _ = Box::from_raw(metric_ptr);
    }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ten_metric_counter_inc(metric_ptr: *mut MetricHandle) {
    debug_assert!(!metric_ptr.is_null(), "Metric pointer is null");

    if metric_ptr.is_null() {
        return;
    }

    let metric = unsafe { &mut *metric_ptr };
    match metric {
        MetricHandle::Counter(ref counter) => {
            counter.inc();
        }
        MetricHandle::CounterVec { vec, label_values } => {
            let label_refs: Vec<&str> =
                label_values.iter().map(|s| s.as_str()).collect();
            if let Ok(counter) = vec.get_metric_with_label_values(&label_refs) {
                counter.inc();
            }
        }
        _ => {}
    }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ten_metric_counter_add(
    metric_ptr: *mut MetricHandle,
    value: f64,
) {
    debug_assert!(!metric_ptr.is_null(), "Metric pointer is null");

    if metric_ptr.is_null() {
        return;
    }

    let metric = unsafe { &mut *metric_ptr };
    match metric {
        MetricHandle::Counter(ref counter) => {
            counter.inc_by(value);
        }
        MetricHandle::CounterVec { vec, label_values } => {
            let label_refs: Vec<&str> =
                label_values.iter().map(|s| s.as_str()).collect();
            if let Ok(counter) = vec.get_metric_with_label_values(&label_refs) {
                counter.inc_by(value);
            }
        }
        _ => {}
    }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ten_metric_gauge_set(
    metric_ptr: *mut MetricHandle,
    value: f64,
) {
    debug_assert!(!metric_ptr.is_null(), "Metric pointer is null");

    if metric_ptr.is_null() {
        return;
    }

    let metric = unsafe { &mut *metric_ptr };
    match metric {
        MetricHandle::Gauge(ref gauge) => {
            gauge.set(value);
        }
        MetricHandle::GaugeVec { vec, label_values } => {
            let label_refs: Vec<&str> =
                label_values.iter().map(|s| s.as_str()).collect();
            if let Ok(gauge) = vec.get_metric_with_label_values(&label_refs) {
                gauge.set(value);
            }
        }
        _ => {}
    }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ten_metric_gauge_inc(metric_ptr: *mut MetricHandle) {
    debug_assert!(!metric_ptr.is_null(), "Metric pointer is null");

    if metric_ptr.is_null() {
        return;
    }

    let metric = unsafe { &mut *metric_ptr };
    match metric {
        MetricHandle::Gauge(ref gauge) => {
            gauge.inc();
        }
        MetricHandle::GaugeVec { vec, label_values } => {
            let label_refs: Vec<&str> =
                label_values.iter().map(|s| s.as_str()).collect();
            if let Ok(gauge) = vec.get_metric_with_label_values(&label_refs) {
                gauge.inc();
            }
        }
        _ => {}
    }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ten_metric_gauge_dec(metric_ptr: *mut MetricHandle) {
    debug_assert!(!metric_ptr.is_null(), "Metric pointer is null");

    if metric_ptr.is_null() {
        return;
    }

    let metric = unsafe { &mut *metric_ptr };
    match metric {
        MetricHandle::Gauge(ref gauge) => {
            gauge.dec();
        }
        MetricHandle::GaugeVec { vec, label_values } => {
            let label_refs: Vec<&str> =
                label_values.iter().map(|s| s.as_str()).collect();
            if let Ok(gauge) = vec.get_metric_with_label_values(&label_refs) {
                gauge.dec();
            }
        }
        _ => {}
    }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ten_metric_histogram_observe(
    metric_ptr: *mut MetricHandle,
    value: f64,
) {
    debug_assert!(!metric_ptr.is_null(), "Metric pointer is null");

    if metric_ptr.is_null() {
        return;
    }

    let metric = unsafe { &mut *metric_ptr };
    match metric {
        MetricHandle::Histogram(ref histogram) => {
            histogram.observe(value);
        }
        MetricHandle::HistogramVec { vec, label_values } => {
            let label_refs: Vec<&str> =
                label_values.iter().map(|s| s.as_str()).collect();
            if let Ok(histogram) = vec.get_metric_with_label_values(&label_refs)
            {
                histogram.observe(value);
            }
        }
        _ => {}
    }
}
