use std::ffi::{c_char, CStr};
use std::slice::from_raw_parts;
use std::str::from_utf8;

use crate::errors::new_nixide_error;
use crate::NixideResult;

pub trait CCharPtrExt {
    fn to_utf8_string(self) -> NixideResult<String>;

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
        let bytes = unsafe { from_raw_parts(self.cast::<u8>(), n as usize) };
        match from_utf8(bytes) {
            Ok(s) => Ok(s.to_string()),
            Err(_) => Err(new_nixide_error!(StringNotUtf8)),
        }
    }
}
