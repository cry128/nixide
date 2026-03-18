use std::ffi::CString;
use std::ptr::NonNull;

use super::{FlakeReference, FlakeSettings};
use crate::sys;
use crate::{ErrorContext, NixErrorCode};

#[derive(Debug, Clone)]
pub enum FlakeLockMode {
    /// Configures [LockedFlake::lock] to make incremental changes to the lock file as needed. Changes are written to file.
    WriteAsNeeded,

    /// Like [FlakeLockMode::WriteAsNeeded], but does not write to the lock file.
    Virtual,

    /// Make [LockedFlake::lock] check if the lock file is up to date. If not, an error is returned.
    Check,
}

/// Parameters that affect the locking of a flake.
pub struct FlakeLockFlags {
    pub(crate) inner: NonNull<sys::nix_flake_lock_flags>,
}
impl Drop for FlakeLockFlags {
    fn drop(&mut self) {
        unsafe {
            sys::nix_flake_lock_flags_free(self.as_ptr());
        }
    }
}
impl FlakeLockFlags {
    // XXX: TODO: what is the default FlakeLockMode?
    pub fn new(settings: &FlakeSettings) -> Result<Self, NixErrorCode> {
        ErrorContext::new().and_then(|ctx| {
            NonNull::new(unsafe { sys::nix_flake_lock_flags_new(ctx.as_ptr(), settings.as_ptr()) })
                .ok_or(NixErrorCode::NulError {
                    location: "nix_flake_lock_flags_new",
                })
                .map(|inner| FlakeLockFlags { inner })
        })
    }

    pub(crate) fn as_ptr(&self) -> *mut sys::nix_flake_lock_flags {
        self.inner.as_ptr()
    }

    pub fn set_lock_mode(&mut self, mode: &FlakeLockMode) -> Result<(), NixErrorCode> {
        ErrorContext::new().and_then(|ctx| unsafe {
            NixErrorCode::from(
                match mode {
                    FlakeLockMode::WriteAsNeeded => {
                        sys::nix_flake_lock_flags_set_mode_write_as_needed(
                            ctx.as_ptr(),
                            self.as_ptr(),
                        )
                    }
                    FlakeLockMode::Virtual => {
                        sys::nix_flake_lock_flags_set_mode_virtual(ctx.as_ptr(), self.as_ptr())
                    }
                    FlakeLockMode::Check => {
                        sys::nix_flake_lock_flags_set_mode_check(ctx.as_ptr(), self.as_ptr())
                    }
                },
                "nix_flake_lock_flags_set_mode_check",
            )
        })
    }

    /// Configures [LockedFlake::lock] to make incremental changes to the lock file as needed. Changes are written to file.
    // pub fn set_mode_write_as_needed(&mut self) -> Result<(), NixErrorCode> {
    //     ErrorContext::new().and_then(|ctx| {
    //         NixErrorCode::from(
    //             unsafe {
    //                 sys::nix_flake_lock_flags_set_mode_write_as_needed(ctx.as_ptr(), self.as_ptr())
    //             },
    //             "nix_flake_lock_flags_set_mode_write_as_needed",
    //         )
    //     })
    // }

    /// Make [LockedFlake::lock] check if the lock file is up to date. If not, an error is returned.
    // pub fn set_mode_check(&mut self) -> Result<(), NixErrorCode> {
    //     ErrorContext::new().and_then(|ctx| {
    //         NixErrorCode::from(
    //             unsafe { sys::nix_flake_lock_flags_set_mode_check(ctx.as_ptr(), self.as_ptr()) },
    //             "nix_flake_lock_flags_set_mode_check",
    //         )
    //     })
    // }

    /// Like `set_mode_write_as_needed`, but does not write to the lock file.
    // pub fn set_mode_virtual(&mut self) -> Result<(), NixErrorCode> {
    //     ErrorContext::new().and_then(|ctx| {
    //         NixErrorCode::from(
    //             unsafe { sys::nix_flake_lock_flags_set_mode_virtual(ctx.as_ptr(), self.as_ptr()) },
    //             "nix_flake_lock_flags_set_mode_virtual",
    //         )
    //     })
    // }

    /// Adds an input override to the lock file that will be produced.
    /// The [LockedFlake::lock] operation will not write to the lock file.
    ///
    /// # Warning
    ///
    /// Calling this function will implicitly set the [FlakeLockMode] to
    /// [FlakeLockMode::Virtual] if `self.mode` is not [FlakeLockMode::Check].
    ///
    /// # Arguments
    ///
    ///  * `path` - The input name/path to override (must not be empty)
    ///  * `flake_ref` - The flake reference to use as the override
    pub fn override_input(
        &mut self,
        path: &str,
        flakeref: &FlakeReference,
    ) -> Result<(), NixErrorCode> {
        let input_path = NixErrorCode::from_nulerror(
            CString::new(path),
            "nixide::FlakeLockArgs::override_input",
        )?;

        ErrorContext::new().and_then(|ctx| unsafe {
            NixErrorCode::from(
                unsafe {
                    sys::nix_flake_lock_flags_add_input_override(
                        ctx.as_ptr(),
                        self.as_ptr(),
                        input_path.as_ptr(),
                        flakeref.as_ptr(),
                    )
                },
                "nix_flake_lock_flags_add_input_override",
            )
        })
    }
}
