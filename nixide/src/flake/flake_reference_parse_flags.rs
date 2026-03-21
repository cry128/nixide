use std::os::raw::c_char;
use std::ptr::NonNull;

use super::FlakeSettings;
use crate::errors::{new_nixide_error, ErrorContext};
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::NixideError;

/// Parameters for parsing a flake reference.
#[derive(Debug)]
pub struct FlakeReferenceParseFlags {
    pub(crate) ptr: NonNull<sys::nix_flake_reference_parse_flags>,
}

impl Drop for FlakeReferenceParseFlags {
    fn drop(&mut self) {
        unsafe {
            sys::nix_flake_reference_parse_flags_free(self.ptr.as_ptr());
        }
    }
}

impl FlakeReferenceParseFlags {
    pub fn new(settings: &FlakeSettings) -> Result<Self, NixideError> {
        let ctx = ErrorContext::new();
        let ptr =
            unsafe { sys::nix_flake_reference_parse_flags_new(ctx.as_ptr(), settings.as_ptr()) };
        match ctx.peak() {
            Some(err) => Err(err),
            None => NonNull::new(ptr).map_or(Err(new_nixide_error!(NullPtr)), |ptr| {
                Ok(FlakeReferenceParseFlags { ptr })
            }),
        }
    }

    /// Sets the [base directory](https://nix.dev/manual/nix/latest/glossary#gloss-base-directory)
    /// for resolving local flake references.
    pub fn set_base_directory(&mut self, base_directory: &str) -> Result<(), NixideError> {
        let ctx = ErrorContext::new();
        unsafe {
            sys::nix_flake_reference_parse_flags_set_base_directory(
                ctx.as_ptr(),
                self.as_ptr(),
                base_directory.as_ptr() as *const c_char,
                base_directory.len(),
            )
        };
        match ctx.peak() {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    pub fn as_ptr(&self) -> *mut sys::nix_flake_reference_parse_flags {
        self.ptr.as_ptr()
    }
}
