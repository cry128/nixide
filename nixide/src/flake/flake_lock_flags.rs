use std::ptr::NonNull;

use super::{FlakeReference, FlakeSettings};
use crate::NixideResult;
use crate::errors::ErrorContext;
use crate::stdext::AsCPtr as _;
use crate::sys;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr;

#[derive(Debug, Clone, Copy)]
pub enum FlakeLockMode {
    /// Configures [LockedFlake::lock] to make incremental changes to the lock file as needed. Changes are written to file.
    ///
    WriteAsNeeded,

    /// Like [FlakeLockMode::WriteAsNeeded], but does not write to the lock file.
    ///
    Virtual,

    /// Make [LockedFlake::lock] check if the lock file is up to date. If not, an error is returned.
    ///
    Check,
}

/// Parameters that affect the locking of a flake.
pub struct FlakeLockFlags {
    pub(crate) inner: NonNull<sys::NixFlakeLockFlags>,
}

// impl Clone for FlakeLockFlags {
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

impl Drop for FlakeLockFlags {
    fn drop(&mut self) {
        unsafe {
            sys::nix_flake_lock_flags_free(self.as_ptr());
        }
    }
}

impl AsInnerPtr<sys::NixFlakeLockFlags> for FlakeLockFlags {
    #[inline]
    unsafe fn as_ptr(&self) -> *mut sys::NixFlakeLockFlags {
        self.inner.as_ptr()
    }

    #[inline]
    unsafe fn as_ref(&self) -> &sys::NixFlakeLockFlags {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    unsafe fn as_mut(&mut self) -> &mut sys::NixFlakeLockFlags {
        unsafe { self.inner.as_mut() }
    }
}

impl FlakeLockFlags {
    // XXX: TODO: what is the default FlakeLockMode?
    pub fn new(settings: &FlakeSettings) -> NixideResult<Self> {
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_flake_lock_flags_new(ctx.as_ptr(), settings.as_ptr())
        })?;

        Ok(FlakeLockFlags { inner })
    }

    pub fn set_mode(&mut self, mode: &FlakeLockMode) -> NixideResult<()> {
        wrap::nix_fn!(|ctx: &ErrorContext| {
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
        })
    }

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
    ///  * `flakeref` - The flake reference to use as the override
    pub fn override_input(&mut self, path: &str, flakeref: &FlakeReference) -> NixideResult<()> {
        let input_path = path.as_c_ptr()?;

        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_flake_lock_flags_add_input_override(
                ctx.as_ptr(),
                self.as_ptr(),
                input_path,
                flakeref.as_ptr(),
            );
        })
    }
}
