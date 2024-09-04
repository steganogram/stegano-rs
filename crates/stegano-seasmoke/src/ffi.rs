use crate::ffi_utils::ByteBuffer;
use std::ffi::{c_char, CStr};

/// # Safety
/// This function is unsafe because it dereferences the password raw pointer and assumes that the data is valid.
/// It returns a null pointer in any case of error
#[no_mangle]
pub unsafe extern "C" fn decrypt_data(
    password: *const c_char,
    data: *const u8,
    data_len: usize,
) -> *const ByteBuffer {
    let password = match unsafe { CStr::from_ptr(password).to_str() } {
        Ok(password) => password,
        Err(_) => return std::ptr::null_mut(),
    };
    let data = unsafe { std::slice::from_raw_parts(data, data_len) };

    let decipher_data = match crate::decrypt_data(password, data) {
        Ok(data) => data,
        Err(_) => return std::ptr::null_mut(),
    };

    let buffer = ByteBuffer::from_vec(decipher_data);

    Box::into_raw(Box::new(buffer))
}

/// # Safety
/// This function is unsafe because it dereferences the password raw pointer and assumes that the data is valid.
/// It returns a null pointer in any case of error
#[no_mangle]
pub unsafe extern "C" fn encrypt_data(
    password: *const c_char,
    data: *const u8,
    data_len: usize,
) -> *const ByteBuffer {
    let password = match unsafe { CStr::from_ptr(password).to_str() } {
        Ok(password) => password,
        Err(_) => return std::ptr::null_mut(),
    };
    let data = unsafe { std::slice::from_raw_parts(data, data_len) };

    let cipher_data = match crate::encrypt_data(password, data) {
        Ok(data) => data,
        Err(_) => return std::ptr::null_mut(),
    };

    let buffer = ByteBuffer::from_vec(cipher_data);

    Box::into_raw(Box::new(buffer))
}
