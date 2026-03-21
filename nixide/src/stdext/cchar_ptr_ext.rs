use std::ffi::{c_char, CStr};
use std::slice::from_raw_parts;
use std::str::{from_utf8, Utf8Error};

pub trait CCharPtrExt {
    fn to_utf8_string(self) -> Result<String, Option<Utf8Error>>;

    fn to_utf8_string_n(self, n: usize) -> Result<String, Option<Utf8Error>>;
}

impl CCharPtrExt for *const c_char {
    fn to_utf8_string(self) -> Result<String, Option<Utf8Error>> {
        if self.is_null() {
            return Err(None);
        }
        let cstr = unsafe { CStr::from_ptr(self) };
        match cstr.to_str() {
            Ok(s) => Ok(s.to_owned()),
            Err(err) => Err(Some(err)),
        }
    }

    fn to_utf8_string_n(self, n: usize) -> Result<String, Option<Utf8Error>> {
        if self.is_null() || n == 0 {
            return Err(None);
        }
        let bytes = unsafe { from_raw_parts(self.cast::<u8>(), n as usize) };
        from_utf8(bytes).map(str::to_string).map_err(Some)
    }
}
