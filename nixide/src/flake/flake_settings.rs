use std::ptr::NonNull;

use crate::errors::{new_nixide_error, ErrorContext};
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::{EvalStateBuilder, NixideError};

/// Store settings for the flakes feature.
pub struct FlakeSettings {
    pub(crate) inner: NonNull<sys::nix_flake_settings>,
}

impl AsInnerPtr<sys::nix_flake_settings> for FlakeSettings {
    unsafe fn as_ptr(&self) -> *mut sys::nix_flake_settings {
        self.inner.as_ptr()
    }
}

impl FlakeSettings {
    pub fn new() -> Result<Self, NixideError> {
        let ctx = ErrorContext::new();
        let opt = NonNull::new(unsafe { sys::nix_flake_settings_new(ctx.as_ptr()) });

        match ctx.peak() {
            Some(err) => Err(err),
            None => match opt {
                Some(inner) => Ok(FlakeSettings { inner }),
                None => Err(new_nixide_error!(NullPtr)),
            },
        }
    }

    pub(super) fn add_to_eval_state_builder(
        &self,
        builder: &mut EvalStateBuilder,
    ) -> Result<(), NixideError> {
        let ctx = ErrorContext::new();
        unsafe {
            sys::nix_flake_settings_add_to_eval_state_builder(
                ctx.as_ptr(),
                self.as_ptr(),
                builder.as_ptr(),
            )
        };
        match ctx.peak() {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}

impl Drop for FlakeSettings {
    fn drop(&mut self) {
        unsafe {
            sys::nix_flake_settings_free(self.as_ptr());
        }
    }
}
