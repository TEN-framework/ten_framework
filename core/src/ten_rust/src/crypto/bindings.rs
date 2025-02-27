//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//

use std::ptr;
use std::{ffi::CStr, os::raw::c_char};

use crate::crypto;

use super::EncryptionAlgorithm;

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ten_encrypt_algorithm_create(
    algorithm: *const c_char,
    params: *const c_char,
) -> *mut EncryptionAlgorithm {
    let algorithm_cstr = unsafe { CStr::from_ptr(algorithm) };
    let params_cstr = unsafe { CStr::from_ptr(params) };

    let algorithm_str = algorithm_cstr.to_str().unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        ""
    });
    if algorithm_str.is_empty() {
        return ptr::null_mut();
    }

    let params_str = params_cstr.to_str().unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        ""
    });
    if params_str.is_empty() {
        return ptr::null_mut();
    }

    let algorithm =
        crypto::create_encryption_algorithm(algorithm_str, params_str);

    match algorithm {
        Ok(algorithm) => Box::into_raw(Box::new(algorithm)),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn ten_encrypt_inplace(
    algorithm_ptr: *mut EncryptionAlgorithm,
    data: *mut u8,
    data_len: i32,
) -> bool {
    if algorithm_ptr.is_null() || data.is_null() || data_len <= 0 {
        return false;
    }

    let algorithm = &mut *algorithm_ptr;
    let data_slice = std::slice::from_raw_parts_mut(data, data_len as usize);
    algorithm.encrypt(data_slice);
    true
}
