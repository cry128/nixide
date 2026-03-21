use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_uint, c_void};
use std::path::PathBuf;

use crate::errors::{ErrorContext, NixideError};
use crate::util::CCharPtrNixExt;

pub fn wrap_libnix_string_callback<F>(callback: F) -> Result<String, NixideError>
where
    F: FnOnce(
        &ErrorContext,
        unsafe extern "C" fn(*const c_char, c_uint, *mut c_void),
        *mut c_void,
    ) -> i32,
{
    // Callback to receive the string
    unsafe extern "C" fn wrapper_callback(start: *const c_char, n: c_uint, user_data: *mut c_void) {
        let result = unsafe { &mut *(user_data as *mut Result<String, NixideError>) };

        *result = start.to_utf8_string_sized(n as usize);
    }

    let ctx = ErrorContext::new();
    let mut user_data: MaybeUninit<Result<String, NixideError>> = MaybeUninit::uninit();

    callback(
        &ctx,
        wrapper_callback,
        user_data.as_mut_ptr() as *mut c_void,
    );
    if let Some(err) = ctx.peak() {
        return Err(err);
    }

    unsafe { user_data.assume_init() }
}

pub fn wrap_libnix_pathbuf_callback<F>(callback: F) -> Result<PathBuf, NixideError>
where
    F: FnOnce(
        &ErrorContext,
        unsafe extern "C" fn(*const c_char, c_uint, *mut c_void),
        *mut c_void,
    ) -> i32,
{
    wrap_libnix_string_callback(callback).map(PathBuf::from)
}
