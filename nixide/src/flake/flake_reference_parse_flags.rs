use std::ffi::c_char;
use std::ptr::NonNull;

use super::FlakeSettings;
use crate::NixideResult;
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr;

/// Parameters for parsing a flake reference.
#[derive(Debug)]
pub struct FlakeReferenceParseFlags {
    inner: NonNull<sys::NixFlakeReferenceParseFlags>,
}

// impl Clone for FlakeReferenceParseFlags {
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

impl Drop for FlakeReferenceParseFlags {
    fn drop(&mut self) {
        unsafe {
            sys::nix_flake_reference_parse_flags_free(self.inner.as_ptr());
        }
    }
}

impl AsInnerPtr<sys::NixFlakeReferenceParseFlags> for FlakeReferenceParseFlags {
    #[inline]
    unsafe fn as_ptr(&self) -> *mut sys::NixFlakeReferenceParseFlags {
        self.inner.as_ptr()
    }

    #[inline]
    unsafe fn as_ref(&self) -> &sys::NixFlakeReferenceParseFlags {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    unsafe fn as_mut(&mut self) -> &mut sys::NixFlakeReferenceParseFlags {
        unsafe { self.inner.as_mut() }
    }
}

impl FlakeReferenceParseFlags {
    pub fn new(settings: &FlakeSettings) -> NixideResult<Self> {
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_flake_reference_parse_flags_new(ctx.as_ptr(), settings.as_ptr())
        })?;

        Ok(Self { inner })
    }

    /// Sets the [base directory](https://nix.dev/manual/nix/latest/glossary#gloss-base-directory)
    /// for resolving local flake references.
    pub fn set_base_directory(&mut self, base_directory: &str) -> NixideResult<()> {
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_flake_reference_parse_flags_set_base_directory(
                ctx.as_ptr(),
                self.as_ptr(),
                base_directory.as_ptr() as *const c_char,
                base_directory.len(),
            );
        })
    }
}
