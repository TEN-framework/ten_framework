//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
mod bindings;

use std::{
    ffi::c_char,
    sync::{Arc, Mutex},
};

use anyhow::{Ok, Result};

use bindings::ten_value_t;

/// `TenValueRawPointer` acts as a proxy for the raw pointer to `ten_value_t`
/// in C. When the Rust object bound to this pointer is dropped, we need to
/// ensure that the raw pointer is properly destroyed. However, the `Drop` trait
/// cannot be directly implemented on a raw pointer.
///
/// This wrapper provides safe memory management for the C-allocated value
/// object, ensuring proper cleanup when the Rust object goes out of scope.
#[derive(Debug)]
struct TenValueRawPointer {
    /// The raw pointer to the C-allocated value object.
    /// Protected by a Mutex to allow safe sharing between threads.
    raw_ptr: Mutex<*const ten_value_t>,
}

// Explicitly mark this type as thread-safe since we're handling raw pointers.
// While raw pointers are not automatically Send or Sync in Rust, we can
// safely implement these traits because:
//
// 1. The raw_ptr is protected by a Mutex, ensuring exclusive access during
//    operations.
// 2. The pointer is effectively read-only after initialization.
// 3. The actual mutation only happens during Drop, which has exclusive access.
//
// This allows TenValue to be safely shared between threads in the application.
unsafe impl Send for TenValueRawPointer {}
unsafe impl Sync for TenValueRawPointer {}

impl Drop for TenValueRawPointer {
    fn drop(&mut self) {
        // Acquire lock on the pointer to ensure exclusive access during
        // cleanup.
        let ptr = self.raw_ptr.lock().unwrap();

        // Only attempt to free non-null pointers to avoid undefined behavior.
        if !ptr.is_null() {
            unsafe {
                // Call the C function to properly free the value resources.
                bindings::ten_value_destroy_proxy(*ptr);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TenValue {
    // TenValue represents a value object that holds TEN data values. It
    // needs to implement Clone for use in various contexts, and must be
    // thread-safe for use in the tman designer where it will be shared between
    // threads. We use Arc to wrap the raw pointer to ensure safe concurrent
    // access and proper reference counting.
    inner: Arc<TenValueRawPointer>,
}

impl TenValue {
    /// Creates a new TenValue instance from a raw C pointer to a ten_value_t.
    ///
    /// This constructor takes ownership of the provided raw pointer and ensures
    /// it will be properly cleaned up when the TenValue is dropped.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - The raw_ptr is a valid pointer to a ten_value_t structure.
    /// - The raw_ptr was created by the C API
    ///   (ten_value_create_from_json_str_proxy).
    /// - Ownership of the pointer is transferred to this TenValue instance.
    pub fn new(raw_ptr: *const ten_value_t) -> Self {
        Self {
            // The raw_ptr is wrapped in a Mutex and Arc to ensure thread-safety
            // and proper reference counting. While the raw_ptr itself is not
            // inherently Send or Sync, our wrapper provides safe concurrent
            // access.
            inner: Arc::new(TenValueRawPointer {
                raw_ptr: Mutex::new(raw_ptr),
            }),
        }
    }

    pub fn as_ptr(&self) -> *const ten_value_t {
        *self.inner.raw_ptr.lock().unwrap()
    }
}

pub fn create_value_from_json(
    value_json: &serde_json::Value,
) -> Result<TenValue> {
    let value_str = serde_json::to_string(value_json)?;
    create_value_from_json_str(&value_str)
}

pub fn create_value_from_json_str(value_json_str: &str) -> Result<TenValue> {
    let mut err_msg: *mut c_char = std::ptr::null_mut();
    let value_ptr: *mut ten_value_t;
    unsafe {
        let c_value_str = std::ffi::CString::new(value_json_str)?;
        value_ptr = bindings::ten_value_create_from_json_str_proxy(
            c_value_str.as_ptr(),
            &mut err_msg as *mut _ as *mut *const c_char,
        );
    }

    if value_ptr.is_null() {
        // Failed to create ten_value_t in C world.
        unsafe {
            let rust_err_msg =
                std::ffi::CStr::from_ptr(err_msg).to_str()?.to_owned();
            // The err_msg is allocated by the C code, so we need to free it.
            libc::free(err_msg as *mut libc::c_void);

            return Err(anyhow::anyhow!(rust_err_msg));
        }
    }

    Ok(TenValue::new(value_ptr))
}
