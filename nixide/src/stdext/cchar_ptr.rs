use std::ffi::{CStr, c_char};
use std::slice::from_raw_parts;
use std::str::from_utf8;

use crate::NixideResult;
use crate::errors::new_nixide_error;

pub trait AsCPtr<T> {
    #[allow(unused)]
    fn as_c_ptr(&self) -> NixideResult<*const T>;

    #[allow(unused)]
    fn into_c_ptr(self) -> NixideResult<*mut T>;
}

impl<T> AsCPtr<c_char> for T
where
    T: AsRef<str>,
{
    fn as_c_ptr(&self) -> NixideResult<*const c_char> {
        match CStr::from_bytes_until_nul(self.as_ref().as_bytes()) {
            Ok(s) => Ok(s.as_ptr()),
            Err(_) => Err(new_nixide_error!(StringNulByte)),
        }
    }

    fn into_c_ptr(self) -> NixideResult<*mut c_char> {
        match CStr::from_bytes_until_nul(self.as_ref().as_bytes()) {
            Ok(s) => Ok(s.as_ptr().cast_mut()),
            Err(_) => Err(new_nixide_error!(StringNulByte)),
        }
    }
}

pub trait CCharPtrExt {
    #[allow(unused)]
    fn to_utf8_string(self) -> NixideResult<String>;

    #[allow(unused)]
    fn to_utf8_string_n(self, n: usize) -> NixideResult<String>;
}

impl CCharPtrExt for *const c_char {
    fn to_utf8_string(self) -> NixideResult<String> {
        if self.is_null() {
            return Err(new_nixide_error!(NullPtr));
        }
        let cstr = unsafe { CStr::from_ptr(self) };
        match cstr.to_str() {
            Ok(s) => Ok(s.to_owned()),
            Err(_) => Err(new_nixide_error!(StringNotUtf8)),
        }
    }

    fn to_utf8_string_n(self, n: usize) -> NixideResult<String> {
        if self.is_null() || n == 0 {
            return Err(new_nixide_error!(NullPtr));
        }
        let bytes = unsafe { from_raw_parts(self as *const u8, n as usize) };
        match from_utf8(bytes) {
            Ok(s) => Ok(s.to_string()),
            Err(_) => Err(new_nixide_error!(StringNotUtf8)),
        }
    }
}
