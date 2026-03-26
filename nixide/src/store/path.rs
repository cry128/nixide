use std::ffi::{c_void, CString};
use std::path::PathBuf;
use std::ptr::NonNull;

use super::Store;
use crate::errors::{new_nixide_error, ErrorContext};
use crate::sys;
use crate::util::panic_issue_call_failed;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr;
use crate::NixideResult;

/// A path in the Nix store.
///
/// Represents a store path that can be realized, queried, or manipulated.
///
pub struct StorePath {
    pub(crate) inner: NonNull<sys::StorePath>,
}

impl AsInnerPtr<sys::StorePath> for StorePath {
    #[inline]
    unsafe fn as_ptr(&self) -> *mut sys::StorePath {
        self.inner.as_ptr()
    }

    #[inline]
    unsafe fn as_ref(&self) -> &sys::StorePath {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    unsafe fn as_mut(&mut self) -> &mut sys::StorePath {
        unsafe { self.inner.as_mut() }
    }
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
    pub fn parse(store: &Store, path: &str) -> NixideResult<Self> {
        let c_path = CString::new(path).or(Err(new_nixide_error!(StringNulByte)))?;

        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_store_parse_path(ctx.as_ptr(), store.as_ptr(), c_path.as_ptr())
        })?;

        Ok(Self { inner })
    }

    pub fn fake_path(store: &Store) -> NixideResult<Self> {
        Self::parse(store, "/nix/store/00000000000000000000000000000000-fake")
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
    pub fn name(&self) -> NixideResult<String> {
        wrap::nix_string_callback!(|callback, userdata: *mut __UserData, _| unsafe {
            sys::nix_store_path_name(self.inner.as_ptr(), Some(callback), userdata as *mut c_void);
            // NOTE: nix_store_path_name doesn't return nix_err, so we force it to return successfully
            // XXX: NOTE: now `nix_string_callback` is a macro this isn't necessary
            // sys::nix_err_NIX_OK
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
    pub fn real_path(&self, store: &Store) -> NixideResult<PathBuf> {
        wrap::nix_pathbuf_callback!(
            |callback, userdata: *mut __UserData, ctx: &ErrorContext| unsafe {
                sys::nix_store_real_path(
                    ctx.as_ptr(),
                    store.inner.as_ptr(),
                    self.as_ptr(),
                    Some(callback),
                    userdata as *mut c_void,
                )
            }
        )
    }

    /// Check if a [StorePath] is valid (i.e. that its corresponding store object
    /// and its closure of references exists in the store).
    ///
    /// # Arguments
    ///
    /// * `store` - The store containing the path
    ///
    pub fn is_valid(&self, store: &Store) -> bool {
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_store_is_valid_path(ctx.as_ptr(), store.as_ptr(), self.as_ptr())
        })
        .is_ok()
    }
}

impl Clone for StorePath {
    fn clone(&self) -> Self {
        let inner = wrap::nix_ptr_fn!(|_| unsafe { sys::nix_store_path_clone(self.as_ptr()) })
            .unwrap_or_else(|_| {
                panic_issue_call_failed!("nix_store_path_clone returned None for valid path")
            });

        StorePath { inner }
    }
}

impl Drop for StorePath {
    fn drop(&mut self) {
        unsafe {
            sys::nix_store_path_free(self.as_ptr());
        }
    }
}

// SAFETY: StorePath can be shared between threads
unsafe impl Send for StorePath {}
unsafe impl Sync for StorePath {}
