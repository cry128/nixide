use std::ffi::CString;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::sync::Arc;

use super::Store;
use crate::errors::{new_nixide_error, ErrorContext};
use crate::util::bindings::{wrap_libnix_pathbuf_callback, wrap_libnix_string_callback};
use crate::util::wrappers::AsInnerPtr;
use crate::NixideError;

use nixide_sys::{self as sys, nix_err_NIX_OK};

/// A path in the Nix store.
///
/// Represents a store path that can be realized, queried, or manipulated.
pub struct StorePath {
    pub(crate) inner: NonNull<sys::StorePath>,
}

impl StorePath {
    /// Parse a store path string into a StorePath.
    ///
    /// # Arguments
    ///
    /// * `store` - The store containing the path
    /// * `path` - The store path string (e.g., "/nix/store/...")
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be parsed.
    pub fn parse(store: &Store, path: &str) -> Result<Self, NixideError> {
        let path_cstring = CString::new(path).or(Err(new_nixide_error!(
            InvalidArg,
            "path",
            "contains a `\\0` (NUL) byte".to_owned()
        )))?;

        let ctx = ErrorContext::new();
        let path_ptr = unsafe {
            sys::nix_store_parse_path(ctx.as_ptr(), store.as_ptr(), path_cstring.as_ptr())
        };

        match ctx.peak() {
            Some(err) => Err(err),
            None => match NonNull::new(path_ptr) {
                Some(inner) => Ok(Self { inner }),
                None => Err(new_nixide_error!(NullPtr)),
            },
        }
    }

    /// Get the name component of the store path.
    ///
    /// This returns the name part of the store path (everything after the hash).
    /// For example, for "/nix/store/abc123...-hello-1.0", this returns "hello-1.0".
    ///
    /// # Errors
    ///
    /// Returns an error if the name cannot be retrieved.
    ///
    pub fn name(&self) -> Result<String, NixideError> {
        wrap_libnix_string_callback(|_, callback, user_data| unsafe {
            sys::nix_store_path_name(self.inner.as_ptr(), Some(callback), user_data);

            // NOTE: nix_store_path_name doesn't return nix_err, so we force it to return successfully
            nix_err_NIX_OK
        })
    }

    /// Get the physical location of a store path
    ///
    /// A store may reside at a different location than its `storeDir` suggests.
    /// This situation is called a relocated store.
    ///
    /// Relocated stores are used during NixOS installation, as well as in restricted
    /// computing environments that don't offer a writable `/nix/store`.
    ///
    /// Not all types of stores support this operation.
    ///
    /// # Arguments
    /// * `context` [in]  - Optional, stores error information
    /// * `store` [in]  - nix store reference
    /// * `path` [in]  - the path to get the real path from
    /// * `callback` [in]  - called with the real path
    /// * `user_data` [in]  - arbitrary data, passed to the callback when it's called.
    ///
    /// # Arguments
    ///
    /// * `store` - The store containing the path
    ///
    pub fn real_path(&self, store: &Store) -> Result<PathBuf, NixideError> {
        wrap_libnix_pathbuf_callback(|ctx, callback, user_data| unsafe {
            let err_code = sys::nix_store_real_path(
                ctx.as_ptr(),
                store.inner.as_ptr(),
                self.as_ptr(),
                Some(callback),
                user_data,
            );
            match ctx.pop() {
                Some(err) => Err(err),
                None => Ok(()),
            }

            err_code
        })
    }

    /// Check if a [StorePath] is valid (i.e. that its corresponding store object
    /// and its closure of references exists in the store).
    ///
    /// # Arguments
    ///
    /// * `store` - The store containing the path
    ///
    pub fn is_valid(&self, store: &Store) -> bool {
        let ctx = ErrorContext::new();
        unsafe {
            sys::nix_store_is_valid_path(ctx.as_ptr(), store.inner.as_ptr(), self.inner.as_ptr())
        }

        ctx
    }

    /// Get the raw store path pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure the pointer is used safely.
    pub(crate) unsafe fn as_ptr(&self) -> *mut sys::StorePath {
        self.inner.as_ptr()
    }
}

impl Clone for StorePath {
    fn clone(&self) -> Self {
        // SAFETY: self.inner is valid, nix_store_path_clone creates a new copy
        let cloned_ptr = unsafe { sys::nix_store_path_clone(self.inner.as_ptr()) };

        // This should never fail as cloning a valid path should always succeed
        let inner =
            NonNull::new(cloned_ptr).expect("nix_store_path_clone returned null for valid path");

        StorePath {
            inner,
            _context: Arc::clone(&self._context),
        }
    }
}

impl Drop for StorePath {
    fn drop(&mut self) {
        // SAFETY: We own the store path and it's valid until drop
        unsafe {
            sys::nix_store_path_free(self.inner.as_ptr());
        }
    }
}

// SAFETY: StorePath can be shared between threads
unsafe impl Send for StorePath {}
unsafe impl Sync for StorePath {}
