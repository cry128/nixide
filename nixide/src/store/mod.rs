// XXX: TODO: should I add support for `nix_libstore_init_no_load_config`
// XXX: TODO: add support for nix_realised_string_* family of functions
//   nix_realised_string_get_store_path
//   nix_realised_string_get_store_path_count
//   # nix_store_real_path
//   # nix_store_is_valid_path
//   # nix_store_get_version
//   # nix_store_get_uri
//   # nix_store_get_storedir
//   # nix_store_copy_closure
//   nix_libstore_init_no_load_config

#[cfg(test)]
mod tests;

mod path;
pub use path::*;

use std::ffi::{CStr, CString, NulError};
use std::os::raw::{c_char, c_void};
use std::path::PathBuf;
use std::ptr::NonNull;
use std::result::Result;
use std::sync::Arc;

use super::{ErrorContext, NixErrorCode};
use crate::util::bindings::{wrap_libnix_pathbuf_callback, wrap_libnix_string_callback};
use nixide_sys as sys;

/// Nix store for managing packages and derivations.
///
/// The store provides access to Nix packages, derivations, and store paths.
pub struct Store {
    pub(crate) inner: NonNull<sys::Store>,
    pub(crate) _context: Arc<ErrorContext>,
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
    pub fn open(context: &Arc<ErrorContext>, uri: Option<&str>) -> Result<Self, NixErrorCode> {
        let uri_cstring: CString;
        let uri_ptr = if let Some(uri) = uri {
            uri_cstring = NixErrorCode::from_nulerror(CString::new(uri), "nixide::Store::open")?;
            uri_cstring.as_ptr()
        } else {
            std::ptr::null()
        };

        // SAFETY: context is valid, uri_ptr is either null or valid CString
        let store_ptr =
            unsafe { sys::nix_store_open(context.as_ptr(), uri_ptr, std::ptr::null_mut()) };

        let inner = NonNull::new(store_ptr).ok_or(NixErrorCode::NullPtr {
            location: "nix_store_open",
        })?;

        Ok(Store {
            inner,
            _context: Arc::clone(context),
        })
    }

    /// Get the raw store pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure the pointer is used safely.
    pub(crate) unsafe fn as_ptr(&self) -> *mut sys::Store {
        self.inner.as_ptr()
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
    pub fn realise<F>(
        &self,
        path: &StorePath,
        callback: fn(&str, &StorePath),
    ) -> Result<Vec<(String, StorePath)>, NixErrorCode> {
        // Type alias for our userdata: (outputs vector, context)
        type Userdata = (
            Vec<(String, StorePath)>,
            Arc<ErrorContext>,
            fn(&str, &StorePath),
        );

        // Callback function that will be called for each realized output
        unsafe extern "C" fn realise_callback(
            userdata: *mut c_void,
            out_name_ptr: *const c_char,
            out_path_ptr: *const sys::StorePath,
        ) {
            // SAFETY: userdata is a valid pointer to our (Vec, Arc<Context>) tuple
            let (outputs, context, callback) = unsafe { &mut *(userdata as *mut Userdata) };

            // SAFETY: outname is a valid C string from Nix
            let output_name = if !out_name_ptr.is_null() {
                unsafe { CStr::from_ptr(out_name_ptr).to_string_lossy().into_owned() }
            } else {
                String::from("out") // Default output name
            };

            // SAFETY: out is a valid StorePath pointer from Nix, we need to clone it
            // because Nix owns the original and may free it after the callback
            if !out_path_ptr.is_null() {
                let cloned_path_ptr =
                    unsafe { sys::nix_store_path_clone(out_path_ptr as *mut sys::StorePath) };
                if let Some(inner) = NonNull::new(cloned_path_ptr) {
                    let store_path = StorePath {
                        inner,
                        _context: Arc::clone(context),
                    };

                    callback(output_name.as_ref(), &store_path);

                    outputs.push((output_name, store_path));
                }
            }
        }

        // Create userdata with empty outputs vector and context
        let mut userdata: Userdata = (Vec::new(), Arc::clone(&self._context), callback);
        let userdata_ptr = &mut userdata as *mut Userdata as *mut std::os::raw::c_void;

        // SAFETY: All pointers are valid, callback is compatible with the FFI signature
        // - self._context is valid for the duration of this call
        // - self.inner is valid (checked in Store::open)
        // - path.inner is valid (checked in StorePath::parse)
        // - userdata_ptr points to valid stack memory
        // - realize_callback matches the expected C function signature
        let err = unsafe {
            sys::nix_store_realise(
                self._context.as_ptr(),
                self.inner.as_ptr(),
                path.as_ptr(),
                userdata_ptr,
                Some(realise_callback),
            )
        };

        NixErrorCode::from(err, "nix_store_realise")?;

        // Return the collected outputs
        Ok(userdata.0)
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
    /// # use nixide::{Context, Store};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let ctx = Arc::new(Context::new()?);
    /// let store = Store::open(&ctx, None)?;
    /// let path = store.store_path("/nix/store/...")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn store_path(&self, path: &str) -> Result<StorePath, NixErrorCode> {
        StorePath::parse(&self._context, self, path)
    }

    /// Get the version of a Nix store
    ///
    /// If the store doesn't have a version (like the dummy store), returns None
    pub fn version(&self) -> Result<String, NixErrorCode> {
        wrap_libnix_string_callback("nix_store_get_version", |callback, user_data| unsafe {
            sys::nix_store_get_version(
                self._context.as_ptr(),
                self.inner.as_ptr(),
                Some(callback),
                user_data,
            )
        })
    }

    /// Get the URI of a Nix store
    pub fn uri(&self) -> Result<String, NixErrorCode> {
        wrap_libnix_string_callback("nix_store_get_uri", |callback, user_data| unsafe {
            sys::nix_store_get_uri(
                self._context.as_ptr(),
                self.inner.as_ptr(),
                Some(callback),
                user_data,
            )
        })
    }

    pub fn store_dir(&self) -> Result<PathBuf, NixErrorCode> {
        wrap_libnix_pathbuf_callback("nix_store_get_storedir", |callback, user_data| unsafe {
            sys::nix_store_get_storedir(
                self._context.as_ptr(),
                self.inner.as_ptr(),
                Some(callback),
                user_data,
            )
        })
    }

    pub fn copy_closure_to(
        &self,
        dst_store: &Store,
        store_path: &StorePath,
    ) -> Result<(), NixErrorCode> {
        let err = unsafe {
            sys::nix_store_copy_closure(
                self._context.as_ptr(),
                self.inner.as_ptr(),
                dst_store.inner.as_ptr(),
                store_path.inner.as_ptr(),
            )
        };
        NixErrorCode::from(err, "nix_store_copy_closure")
    }

    pub fn copy_closure_from(
        &self,
        src_store: &Store,
        store_path: &StorePath,
    ) -> Result<(), NixErrorCode> {
        let err = unsafe {
            sys::nix_store_copy_closure(
                self._context.as_ptr(),
                src_store.inner.as_ptr(),
                self.inner.as_ptr(),
                store_path.inner.as_ptr(),
            )
        };
        NixErrorCode::from(err, "nix_store_copy_closure")
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        // SAFETY: We own the store and it's valid until drop
        unsafe {
            sys::nix_store_free(self.inner.as_ptr());
        }
    }
}

// SAFETY: Store can be shared between threads
unsafe impl Send for Store {}
unsafe impl Sync for Store {}
