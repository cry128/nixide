use std::ffi::{c_char, c_void};
use std::ptr::{NonNull, null_mut};

use super::{FetchersSettings, FlakeRefParseFlags, FlakeSettings};
use crate::NixideError;
use crate::errors::{ErrorContext, new_nixide_error};
use crate::sys;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr;

pub struct FlakeRef {
    inner: NonNull<sys::NixFlakeReference>,
    fragment: String,

    fetch_settings: FetchersSettings,
    flake_settings: FlakeSettings,
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

impl Drop for FlakeRef {
    fn drop(&mut self) {
        unsafe {
            sys::nix_flake_reference_free(self.as_ptr());
        }
    }
}

impl AsInnerPtr<sys::NixFlakeReference> for FlakeRef {
    #[inline]
    unsafe fn as_ptr(&self) -> *mut sys::NixFlakeReference {
        self.inner.as_ptr()
    }

    #[inline]
    unsafe fn as_ref(&self) -> &sys::NixFlakeReference {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    unsafe fn as_mut(&mut self) -> &mut sys::NixFlakeReference {
        unsafe { self.inner.as_mut() }
    }
}

impl FlakeRef {
    /// Parse a flake reference from a string.
    /// The string must be a valid flake reference, such as `github:owner/repo`.
    /// It may also be suffixed with a `#` and a fragment, such as `github:owner/repo#something`,
    /// in which case, the returned string will contain the fragment.
    pub fn parse<S: AsRef<str>>(reference: S) -> Result<FlakeRef, NixideError> {
        let fetch_settings = FetchersSettings::new()?;
        let flake_settings = FlakeSettings::new()?;
        let parse_flags = FlakeRefParseFlags::new(&flake_settings)?;

        let mut ptr: *mut sys::NixFlakeReference = null_mut();
        let fragment = wrap::nix_string_callback!(
            |callback, userdata: *mut __UserData, ctx: &ErrorContext| unsafe {
                sys::nix_flake_reference_and_fragment_from_string(
                    ctx.as_ptr(),
                    fetch_settings.as_ptr(),
                    flake_settings.as_ptr(),
                    parse_flags.as_ptr(),
                    reference.as_ref().as_ptr() as *const c_char,
                    reference.as_ref().len(),
                    &mut ptr,
                    Some(callback),
                    userdata as *mut c_void,
                )
            }
        )?;

        match NonNull::new(ptr) {
            Some(inner) => Ok(FlakeRef {
                inner,
                fragment,
                fetch_settings,
                flake_settings,
            }),
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
