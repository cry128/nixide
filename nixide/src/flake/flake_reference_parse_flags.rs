use std::ptr::NonNull;

use super::FlakeSettings;
use crate::sys;
use crate::{ErrorContext, NixErrorCode};

/// Parameters for parsing a flake reference.
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
    pub fn new(settings: &FlakeSettings) -> Result<Self, NixErrorCode> {
        let mut ctx = ErrorContext::new();
        let ptr = unsafe {
            context::check_call!(sys::nix_flake_reference_parse_flags_new(
                &mut ctx,
                settings.ptr
            ))
        }?;
        let ptr = NonNull::new(ptr)
            .context("flake_reference_parse_flags_new unexpectedly returned null")?;
        Ok(FlakeReferenceParseFlags { ptr })
    }
    /// Sets the [base directory](https://nix.dev/manual/nix/latest/glossary#gloss-base-directory)
    /// for resolving local flake references.
    pub fn set_base_directory(&mut self, base_directory: &str) -> Result<(), NixErrorCode> {
        let mut ctx = ErrorContext::new();
        unsafe {
            sys::context::check_call!(sys::nix_flake_reference_parse_flags_set_base_directory(
                &mut ctx,
                self.ptr.as_ptr(),
                base_directory.as_ptr() as *const c_char,
                base_directory.len()
            ))
        }?;
        Ok(())
    }
}
