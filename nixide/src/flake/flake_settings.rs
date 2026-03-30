use std::ptr::NonNull;

use crate::NixideResult;
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr;

/// Store settings for the flakes feature.
pub struct FlakeSettings {
    inner: NonNull<sys::nix_flake_settings>,
}

impl Drop for FlakeSettings {
    fn drop(&mut self) {
        unsafe {
            sys::nix_flake_settings_free(self.as_ptr());
        }
    }
}

impl AsInnerPtr<sys::nix_flake_settings> for FlakeSettings {
    #[inline]
    unsafe fn as_ptr(&self) -> *mut sys::nix_flake_settings {
        self.inner.as_ptr()
    }

    #[inline]
    unsafe fn as_ref(&self) -> &sys::nix_flake_settings {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    unsafe fn as_mut(&mut self) -> &mut sys::nix_flake_settings {
        unsafe { self.inner.as_mut() }
    }
}

impl FlakeSettings {
    pub fn new() -> NixideResult<Self> {
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_flake_settings_new(ctx.as_ptr())
        })?;

        Ok(Self { inner })
    }
}
