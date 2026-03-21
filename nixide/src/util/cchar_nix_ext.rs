use std::ffi::{c_char, CStr};
use std::slice::from_raw_parts;
use std::str::from_utf8;

use crate::errors::{new_nixide_error, NixideError};

pub trait CCharPtrNixExt {
    fn to_utf8_string(self) -> Result<String, NixideError>;

    fn to_utf8_string_sized(self, n: usize) -> Result<String, NixideError>;
}

impl CCharPtrNixExt for *const c_char {
    fn to_utf8_string(self) -> Result<String, NixideError> {
        if self.is_null() {
            return Err(new_nixide_error!(NullPtr));
        }

        let result = unsafe { CStr::from_ptr(self).to_str() };
        match result {
            Ok(msg_str) => Ok(msg_str.to_string()),
            Err(_) => Err(new_nixide_error!(StringNulByte)),
        }
    }

    fn to_utf8_string_sized(self, n: usize) -> Result<String, NixideError> {
        if !self.is_null() && n > 0 {
            let bytes = unsafe { from_raw_parts(self.cast::<u8>(), n as usize) };
            from_utf8(bytes)
                .ok()
                .map(|s| s.to_string())
                .ok_or(new_nixide_error!(NullPtr))
        } else {
            Err(new_nixide_error!(NullPtr))
        }
    }
}
