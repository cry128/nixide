use std::ffi::CString;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::sync::Arc;

use super::Store;
use crate::util::bindings::{wrap_libnix_pathbuf_callback, wrap_libnix_string_callback};
use crate::{ErrorContext, NixErrorCode};
use nixide_sys::{self as sys, nix_err_NIX_OK};

/// A path in the Nix store.
///
/// Represents a store path that can be realized, queried, or manipulated.
pub struct StorePath {
    pub(crate) inner: NonNull<sys::StorePath>,
    pub(crate) _context: Arc<ErrorContext>,
}

impl StorePath {
    /// Parse a store path string into a StorePath.
    ///
    /// # Arguments
    ///
    /// * `context` - The Nix context
    /// * `store` - The store containing the path
    /// * `path` - The store path string (e.g., "/nix/store/...")
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be parsed.
    pub fn parse(
        context: &Arc<ErrorContext>,
        store: &Store,
        path: &str,
    ) -> Result<Self, NixErrorCode> {
        let path_cstring = CString::new(path).or(Err(NixErrorCode::InvalidArg {
            location: "nixide::StorePath::parse",
            reason: "`path` contains NUL char",
        }))?;

        // SAFETY: context, store, and path_cstring are valid
        let path_ptr = unsafe {
            sys::nix_store_parse_path(context.as_ptr(), store.as_ptr(), path_cstring.as_ptr())
        };

        let inner = NonNull::new(path_ptr).ok_or(NixErrorCode::NullPtr {
            location: "nix_store_parse_path",
        })?;

        Ok(Self {
            inner,
            _context: Arc::clone(context),
        })
    }

    /// Get the name component of the store path.
    ///
    /// This returns the name part of the store path (everything after the hash).
    /// For example, for "/nix/store/abc123...-hello-1.0", this returns "hello-1.0".
    ///
    /// # Errors
    ///
    /// Returns an error if the name cannot be retrieved.
    pub fn name(&self) -> Result<String, NixErrorCode> {
        wrap_libnix_string_callback("nix_store_path_name", |callback, user_data| unsafe {
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
    pub fn real_path(&self, store: &Store) -> Result<PathBuf, NixErrorCode> {
        wrap_libnix_pathbuf_callback("nix_store_real_path", |callback, user_data| unsafe {
            sys::nix_store_real_path(
                self._context.as_ptr(),
                store.inner.as_ptr(),
                self.inner.as_ptr(),
                Some(callback),
                user_data,
            )
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
        unsafe {
            sys::nix_store_is_valid_path(
                self._context.as_ptr(),
                store.inner.as_ptr(),
                self.inner.as_ptr(),
            )
        }
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
