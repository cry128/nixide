use std::ptr::NonNull;

use crate::sys;
use crate::{ErrorContext, NixErrorCode};

pub(super) struct FetchersSettings {
    pub(super) ptr: NonNull<sys::nix_fetchers_settings>,
}

impl FetchersSettings {
    pub fn new() -> Result<Self, NixErrorCode> {
        let ctx = ErrorContext::new()?;
        let ptr = unsafe { sys::nix_fetchers_settings_new(ctx.as_ptr()) };
        Ok(FetchersSettings {
            ptr: NonNull::new(ptr).ok_or(NixErrorCode::NullPtr {
                location: "fetchers_settings_new",
            })?,
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
