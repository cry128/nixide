use std::ptr::NonNull;

use crate::sys;
use crate::{ErrorContext, EvalStateBuilder, NixErrorCode};

/// Store settings for the flakes feature.
pub struct FlakeSettings {
    pub(crate) inner: NonNull<sys::nix_flake_settings>,
}

impl FlakeSettings {
    pub fn new() -> Result<Self, NixErrorCode> {
        let ctx = ErrorContext::new()?;
        let inner = NonNull::new(unsafe { sys::nix_flake_settings_new(ctx.as_ptr()) }).ok_or(
            NixErrorCode::NullPtr {
                location: "nix_flake_settings_new",
            },
        )?;
        Ok(FlakeSettings { inner })
    }

    fn add_to_eval_state_builder(
        &self,
        builder: &mut EvalStateBuilder,
    ) -> Result<(), NixErrorCode> {
        let ctx = ErrorContext::new()?;
        NixErrorCode::from(
            unsafe {
                sys::nix_flake_settings_add_to_eval_state_builder(
                    ctx.as_ptr(),
                    self.as_ptr(),
                    builder.as_ptr(),
                )
            },
            "nix_flake_settings_add_to_eval_state_builder",
        )?;

        Ok(())
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut sys::nix_flake_settings {
        self.inner.as_ptr()
    }
}

impl Drop for FlakeSettings {
    fn drop(&mut self) {
        unsafe {
            sys::nix_flake_settings_free(self.as_ptr());
        }
    }
}
