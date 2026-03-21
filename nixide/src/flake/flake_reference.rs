use std::os::raw::c_char;
use std::ptr::{null_mut, NonNull};

use super::{FetchersSettings, FlakeReferenceParseFlags, FlakeSettings};
use crate::errors::new_nixide_error;
use crate::sys;
use crate::util::bindings::wrap_libnix_string_callback;
use crate::util::wrappers::AsInnerPtr;
use crate::NixideError;

pub struct FlakeReference {
    pub(crate) ptr: NonNull<sys::nix_flake_reference>,
}

impl Drop for FlakeReference {
    fn drop(&mut self) {
        unsafe {
            sys::nix_flake_reference_free(self.ptr.as_ptr());
        }
    }
}

impl FlakeReference {
    pub fn as_ptr(&self) -> *mut sys::nix_flake_reference {
        self.ptr.as_ptr()
    }

    /// Parse a flake reference from a string.
    /// The string must be a valid flake reference, such as `github:owner/repo`.
    /// It may also be suffixed with a `#` and a fragment, such as `github:owner/repo#something`,
    /// in which case, the returned string will contain the fragment.
    pub fn parse_with_fragment(
        fetch_settings: &FetchersSettings,
        flake_settings: &FlakeSettings,
        flags: &FlakeReferenceParseFlags,
        reference: &str,
    ) -> Result<(FlakeReference, String), NixideError> {
        let mut ptr: *mut sys::nix_flake_reference = null_mut();
        let result = wrap_libnix_string_callback(|ctx, callback, user_data| unsafe {
            sys::nix_flake_reference_and_fragment_from_string(
                ctx.as_ptr(),
                fetch_settings.as_ptr(),
                flake_settings.as_ptr(),
                flags.as_ptr(),
                reference.as_ptr() as *const c_char,
                reference.len(),
                &mut ptr,
                Some(callback),
                user_data,
            )
        });

        match NonNull::new(ptr) {
            Some(ptr) => result.map(|s| (FlakeReference { ptr }, s)),
            None => Err(new_nixide_error!(NullPtr)),
        }
    }
}
