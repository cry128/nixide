use std::ffi::CString;
use std::ptr::NonNull;

use super::{FlakeReference, FlakeSettings};
use crate::errors::new_nixide_error;
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::NixideError;

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
    pub fn new(settings: &FlakeSettings) -> Result<Self, NixideError> {
        let ctx = ErrorContext::new();
        NonNull::new(unsafe { sys::nix_flake_lock_flags_new(ctx.as_ptr(), settings.as_ptr()) })
            .ok_or(new_nixide_error!(NullPtr))
            .map(|inner| FlakeLockFlags { inner })
    }

    pub(crate) fn as_ptr(&self) -> *mut sys::nix_flake_lock_flags {
        self.inner.as_ptr()
    }

    pub fn set_lock_mode(&mut self, mode: &FlakeLockMode) -> Result<(), NixideError> {
        let ctx = ErrorContext::new();
        match mode {
            FlakeLockMode::WriteAsNeeded => unsafe {
                sys::nix_flake_lock_flags_set_mode_write_as_needed(ctx.as_ptr(), self.as_ptr())
            },
            FlakeLockMode::Virtual => unsafe {
                sys::nix_flake_lock_flags_set_mode_virtual(ctx.as_ptr(), self.as_ptr())
            },
            FlakeLockMode::Check => unsafe {
                sys::nix_flake_lock_flags_set_mode_check(ctx.as_ptr(), self.as_ptr())
            },
        };
        match ctx.peak() {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    /// Configures [LockedFlake::lock] to make incremental changes to the lock file as needed. Changes are written to file.
    // pub fn set_mode_write_as_needed(&mut self) -> Result<(), NixideError> {
    //     ErrorContext::new().and_then(|ctx| {
    //         NixideError::from(
    //             unsafe {
    //                 sys::nix_flake_lock_flags_set_mode_write_as_needed(ctx.as_ptr(), self.as_ptr())
    //             },
    //             "nix_flake_lock_flags_set_mode_write_as_needed",
    //         )
    //     })
    // }

    /// Make [LockedFlake::lock] check if the lock file is up to date. If not, an error is returned.
    // pub fn set_mode_check(&mut self) -> Result<(), NixideError> {
    //     ErrorContext::new().and_then(|ctx| {
    //         NixideError::from(
    //             unsafe { sys::nix_flake_lock_flags_set_mode_check(ctx.as_ptr(), self.as_ptr()) },
    //             "nix_flake_lock_flags_set_mode_check",
    //         )
    //     })
    // }

    /// Like `set_mode_write_as_needed`, but does not write to the lock file.
    // pub fn set_mode_virtual(&mut self) -> Result<(), NixideError> {
    //     ErrorContext::new().and_then(|ctx| {
    //         NixideError::from(
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
    ) -> Result<(), NixideError> {
        let input_path = CString::new(path).or_else(|_| Err(new_nixide_error!(StringNulByte)));

        let ctx = ErrorContext::new();
        unsafe {
            sys::nix_flake_lock_flags_add_input_override(
                ctx.as_ptr(),
                self.as_ptr(),
                input_path.as_ptr(),
                flakeref.as_ptr(),
            )
        };
        match ctx.peak() {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}
