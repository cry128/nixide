use std::ptr::NonNull;

use crate::errors::{new_nixide_error, ErrorContext};
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::NixideError;

pub(super) struct FetchersSettings {
    pub(super) ptr: NonNull<sys::nix_fetchers_settings>,
}

impl FetchersSettings {
    pub fn new() -> Result<Self, NixideError> {
        let ctx = ErrorContext::new();
        let ptr = unsafe { sys::nix_fetchers_settings_new(ctx.as_ptr()) };
        Ok(FetchersSettings {
            ptr: NonNull::new(ptr).ok_or(new_nixide_error!(NullPtr))?,
        })
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut sys::nix_fetchers_settings {
        self.ptr.as_ptr()
    }
}

impl Drop for FetchersSettings {
    fn drop(&mut self) {
        unsafe {
            sys::nix_fetchers_settings_free(self.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetchers_settings_new() {
        let _ = FetchersSettings::new().unwrap();
    }
}
