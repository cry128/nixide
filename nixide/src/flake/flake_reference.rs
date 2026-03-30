use std::ffi::c_void;
use std::os::raw::c_char;
use std::ptr::{NonNull, null_mut};

use super::{FetchersSettings, FlakeReferenceParseFlags, FlakeSettings};
use crate::NixideError;
use crate::errors::{ErrorContext, new_nixide_error};
use crate::sys;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr;

// XXX: TODO: rename FlakeReference -> FlakeRef
pub struct FlakeReference {
    inner: NonNull<sys::nix_flake_reference>,
    fragment: String,
}

// impl Clone for FlakeReference {
//     fn clone(&self) -> Self {
//         wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
//             sys::nix_gc_incref(ctx.as_ptr(), self.as_ptr() as *mut c_void);
//         })
//         .unwrap();
//
//         Self {
//             inner: self.inner.clone(),
//             fragment: self.fragment.clone(),
//         }
//     }
// }

impl Drop for FlakeReference {
    fn drop(&mut self) {
        unsafe {
            sys::nix_flake_reference_free(self.as_ptr());
        }
    }
}

impl AsInnerPtr<sys::nix_flake_reference> for FlakeReference {
    #[inline]
    unsafe fn as_ptr(&self) -> *mut sys::nix_flake_reference {
        self.inner.as_ptr()
    }

    #[inline]
    unsafe fn as_ref(&self) -> &sys::nix_flake_reference {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    unsafe fn as_mut(&mut self) -> &mut sys::nix_flake_reference {
        unsafe { self.inner.as_mut() }
    }
}

impl FlakeReference {
    /// Parse a flake reference from a string.
    /// The string must be a valid flake reference, such as `github:owner/repo`.
    /// It may also be suffixed with a `#` and a fragment, such as `github:owner/repo#something`,
    /// in which case, the returned string will contain the fragment.
    pub fn parse(
        fetch_settings: &FetchersSettings,
        flake_settings: &FlakeSettings,
        flags: &FlakeReferenceParseFlags,
        reference: &str,
    ) -> Result<FlakeReference, NixideError> {
        let mut ptr: *mut sys::nix_flake_reference = null_mut();
        let fragment = wrap::nix_string_callback!(
            |callback, userdata: *mut __UserData, ctx: &ErrorContext| unsafe {
                sys::nix_flake_reference_and_fragment_from_string(
                    ctx.as_ptr(),
                    fetch_settings.as_ptr(),
                    flake_settings.as_ptr(),
                    flags.as_ptr(),
                    reference.as_ptr() as *const c_char,
                    reference.len(),
                    &mut ptr,
                    Some(callback),
                    userdata as *mut c_void,
                )
            }
        )?;

        match NonNull::new(ptr) {
            Some(inner) => Ok(FlakeReference { inner, fragment }),
            None => Err(new_nixide_error!(NullPtr)),
        }
    }

    // XXX: TODO: is it possible to get the URI string itself? (minus the fragment part?)
    /// Get a shared reference to the URI fragment part.
    ///
    #[inline]
    #[allow(unused)]
    pub fn fragment(&self) -> &str {
        &self.fragment
    }
}
