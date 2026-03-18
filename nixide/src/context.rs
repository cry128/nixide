use std::ptr::NonNull;

use crate::error::NixError;
use nixide_sys as sys;

/// Nix context for managing library state.
///
/// This is the root object for all Nix operations. It manages the lifetime
/// of the Nix C API context and provides automatic cleanup.
pub struct Context {
    inner: NonNull<sys::nix_c_context>,
}

impl Context {
    /// Create a new Nix context.
    ///
    /// This initializes the Nix C API context and the required libraries.
    ///
    /// # Errors
    ///
    /// Returns an error if context creation or library initialization fails.
    pub fn new() -> Result<Self, NixError> {
        // SAFETY: nix_c_context_create is safe to call
        let ctx_ptr = unsafe { sys::nix_c_context_create() };
        let ctx = Context {
            inner: NonNull::new(ctx_ptr).ok_or(NixError::NullPtr {
                location: "nix_c_context_create",
            })?,
        };

        // Initialize required libraries
        unsafe {
            NixError::from(
                sys::nix_libutil_init(ctx.inner.as_ptr()),
                "nix_libutil_init",
            )?;
            NixError::from(
                sys::nix_libstore_init(ctx.inner.as_ptr()),
                "nix_libstore_init",
            )?;
            NixError::from(
                sys::nix_libexpr_init(ctx.inner.as_ptr()),
                "nix_libexpr_init",
            )?;
        };

        Ok(ctx)
    }

    /// Get the raw context pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure the pointer is used safely.
    pub unsafe fn as_ptr(&self) -> *mut sys::nix_c_context {
        self.inner.as_ptr()
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // SAFETY: We own the context and it's valid until drop
        unsafe {
            sys::nix_c_context_free(self.inner.as_ptr());
        }
    }
}

// SAFETY: Context can be shared between threads
unsafe impl Send for Context {}
unsafe impl Sync for Context {}
