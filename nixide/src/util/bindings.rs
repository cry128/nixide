use std::os::raw::{c_char, c_uint, c_void};
use std::path::PathBuf;

use crate::NixErrorCode;

pub fn wrap_libnix_string_callback<F>(
    name: &'static str,
    callback: F,
) -> Result<String, NixErrorCode>
where
    F: FnOnce(unsafe extern "C" fn(*const c_char, c_uint, *mut c_void), *mut c_void) -> i32,
{
    // Callback to receive the string
    unsafe extern "C" fn wrapper_callback(start: *const c_char, n: c_uint, user_data: *mut c_void) {
        let result = unsafe { &mut *(user_data as *mut Option<String>) };

        if !start.is_null() && n > 0 {
            let bytes = unsafe { std::slice::from_raw_parts(start.cast::<u8>(), n as usize) };
            if let Ok(s) = std::str::from_utf8(bytes) {
                *result = Some(s.to_string());
            }
        }
    }

    let mut result: Option<String> = None;
    let user_data = &mut result as *mut _ as *mut c_void;

    NixErrorCode::from(callback(wrapper_callback, user_data), name)?;
    result.ok_or(NixErrorCode::NullPtr { location: name })
}

pub fn wrap_libnix_pathbuf_callback<F>(
    name: &'static str,
    callback: F,
) -> Result<PathBuf, NixErrorCode>
where
    F: FnOnce(unsafe extern "C" fn(*const c_char, c_uint, *mut c_void), *mut c_void) -> i32,
{
    wrap_libnix_string_callback(name, callback).map(PathBuf::from)
}
