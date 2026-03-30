// XXX: TODO: should I add support for `nix_libstore_init_no_load_config`
//   nix_libstore_init_no_load_config

#[cfg(test)]
mod tests;

mod path;
pub use path::*;

use std::ffi::{c_char, c_void};
use std::path::PathBuf;
use std::ptr::{NonNull, null, null_mut};

use crate::NixideResult;
use crate::errors::ErrorContext;
use crate::stdext::{AsCPtr as _, CCharPtrExt as _};
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::util::{panic_issue_call_failed, wrap};

/// Nix store for managing packages and derivations.
///
/// The store provides access to Nix packages, derivations, and store paths.
///
pub struct Store {
    inner: NonNull<sys::Store>,
}

// impl Clone for Store {
//     fn clone(&self) -> Self {
//         let inner = self.inner.clone();
//
//         wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
//             sys::nix_gc_incref(ctx.as_ptr(), self.as_ptr() as *mut c_void);
//         })
//         .unwrap();
//
//         Self { inner }
//     }
// }

impl AsInnerPtr<sys::Store> for Store {
    #[inline]
    unsafe fn as_ptr(&self) -> *mut sys::Store {
        self.inner.as_ptr()
    }

    #[inline]
    unsafe fn as_ref(&self) -> &sys::Store {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    unsafe fn as_mut(&mut self) -> &mut sys::Store {
        unsafe { self.inner.as_mut() }
    }
}

impl Store {
    /// Open a Nix store.
    ///
    /// # Arguments
    ///
    /// * `context` - The Nix context
    /// * `uri` - Optional store URI (None for default store)
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be opened.
    ///
    pub fn open(uri: &str) -> NixideResult<Self> {
        Self::open_ptr(uri.as_c_ptr()?)
    }

    /// Opens a connection to the default Nix store.
    ///
    pub fn default() -> NixideResult<Self> {
        Self::open_ptr(null())
    }

    #[inline]
    fn open_ptr(uri_ptr: *const c_char) -> NixideResult<Self> {
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            // XXX: TODO: allow args to be parsed instead of just `null_mut`
            sys::nix_store_open(ctx.as_ptr(), uri_ptr, null_mut())
        })?;

        Ok(Store { inner })
    }

    /// Realize a store path.
    ///
    /// This builds/downloads the store path and all its dependencies,
    /// making them available in the local store.
    ///
    /// # Arguments
    ///
    /// * `path` - The store path to realize
    ///
    /// # Returns
    ///
    /// A vector of (output_name, store_path) tuples for each realized output.
    /// For example, a derivation might produce outputs like ("out", path1), ("dev", path2).
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be realized.
    ///
    pub fn realise<F>(
        &self,
        path: &StorePath,
        user_callback: fn(&str, &StorePath),
    ) -> NixideResult<Vec<(String, StorePath)>> {
        wrap::nix_callback!(
            |; userdata: fn(&str, &StorePath);
             output_name_ptr: *const c_char,
             output_path_ptr: *const sys::StorePath|
             -> Vec<(String, StorePath)> {
                // XXX: TODO: test to see if this is ever null ("out" as a default feels unsafe...)
                // NOTE: this also ensures `output_name_ptr` isn't null
                let output_name = output_name_ptr.to_utf8_string().unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

                let inner = wrap::nix_ptr_fn!(|ctx| unsafe {
                    sys::nix_store_path_clone(output_path_ptr as *mut sys::StorePath)
                }).unwrap_or_else(|err| panic_issue_call_failed!("{}", err));
                let store_path = StorePath { inner };

                let callback = unsafe { (*userdata).inner };
                callback(output_name.as_ref(), &store_path);

                (output_name, store_path);
            },
            |callback,
             state: *mut __UserData,
             ctx: &ErrorContext| unsafe {
                // register userdata
                // WARNING: Using `write` instead of assignment via `=`
                // WARNING: to not call `drop` on the old, uninitialized value.
                (&raw mut (*state).inner).write(user_callback);

                sys::nix_store_realise(
                    ctx.as_ptr(),
                    self.as_ptr(),
                    path.as_ptr(),
                    (*state).inner_ptr() as *mut c_void,
                    Some(callback),
                );
            }
        )
    }

    /// Parse a store path string into a StorePath.
    ///
    /// This is a convenience method that wraps `StorePath::parse()`.
    ///
    /// # Arguments
    ///
    /// * `path` - The store path string (e.g., "/nix/store/...")
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be parsed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use nixide::Store;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = Store::open(None)?;
    /// let path = store.store_path("/nix/store/...")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn store_path(&self, path: &str) -> NixideResult<StorePath> {
        StorePath::parse(self, path)
    }

    /// Get the version of a Nix store
    ///
    /// If the store doesn't have a version (like the dummy store), returns None
    ///
    pub fn version(&self) -> NixideResult<String> {
        wrap::nix_string_callback!(
            |callback, userdata: *mut __UserData, ctx: &ErrorContext| unsafe {
                sys::nix_store_get_version(
                    ctx.as_ptr(),
                    self.as_ptr(),
                    Some(callback),
                    userdata as *mut c_void,
                )
            }
        )
    }

    /// Get the URI of a Nix store
    ///
    pub fn uri(&self) -> NixideResult<String> {
        wrap::nix_string_callback!(
            |callback, userdata: *mut __UserData, ctx: &ErrorContext| unsafe {
                sys::nix_store_get_uri(
                    ctx.as_ptr(),
                    self.as_ptr(),
                    Some(callback),
                    userdata as *mut c_void,
                )
            }
        )
    }

    pub fn store_dir(&self) -> NixideResult<PathBuf> {
        wrap::nix_pathbuf_callback!(
            |callback, userdata: *mut __UserData, ctx: &ErrorContext| unsafe {
                sys::nix_store_get_storedir(
                    ctx.as_ptr(),
                    self.as_ptr(),
                    Some(callback),
                    userdata as *mut c_void,
                )
            }
        )
    }

    pub fn copy_closure_to(&self, dst_store: &Store, store_path: &StorePath) -> NixideResult<()> {
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_store_copy_closure(
                ctx.as_ptr(),
                self.as_ptr(),
                dst_store.as_ptr(),
                store_path.as_ptr(),
            );
        })
    }

    pub fn copy_closure_from(&self, src_store: &Store, store_path: &StorePath) -> NixideResult<()> {
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_store_copy_closure(
                ctx.as_ptr(),
                src_store.as_ptr(),
                self.as_ptr(),
                store_path.as_ptr(),
            );
        })
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        unsafe {
            sys::nix_store_free(self.as_ptr());
        }
    }
}
