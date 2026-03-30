use std::ffi::c_void;
use std::ptr::NonNull;

use crate::NixideResult;
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr;

pub struct FetchersSettings {
    inner: NonNull<sys::nix_fetchers_settings>,
}

impl FetchersSettings {
    pub fn new() -> NixideResult<Self> {
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_fetchers_settings_new(ctx.as_ptr())
        })?;

        Ok(Self { inner })
    }
}

// impl Clone for FetchersSettings {
//     fn clone(&self) -> Self {
//         wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
//             sys::nix_gc_incref(ctx.as_ptr(), self.as_ptr() as *mut c_void);
//         })
//         .unwrap();
//
//         Self {
//             inner: self.inner.clone(),
//         }
//     }
// }

impl Drop for FetchersSettings {
    fn drop(&mut self) {
        unsafe {
            sys::nix_fetchers_settings_free(self.as_ptr());
        }
    }
}

impl AsInnerPtr<sys::nix_fetchers_settings> for FetchersSettings {
    #[inline]
    unsafe fn as_ptr(&self) -> *mut sys::nix_fetchers_settings {
        self.inner.as_ptr()
    }

    #[inline]
    unsafe fn as_ref(&self) -> &sys::nix_fetchers_settings {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    unsafe fn as_mut(&mut self) -> &mut sys::nix_fetchers_settings {
        unsafe { self.inner.as_mut() }
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
