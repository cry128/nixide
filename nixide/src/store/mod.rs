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

use std::ffi::{c_char, c_void, CString};
use std::path::PathBuf;
use std::ptr::{null, null_mut, NonNull};
use std::result::Result;

use crate::errors::{new_nixide_error, ErrorContext};
use crate::stdext::CCharPtrExt;
use crate::util::wrap::{self, UserData};
use crate::util::wrappers::AsInnerPtr;
use crate::{NixideError, NixideResult};
use nixide_sys as sys;

/// Nix store for managing packages and derivations.
///
/// The store provides access to Nix packages, derivations, and store paths.
pub struct Store {
    pub(crate) inner: NonNull<sys::Store>,
}

impl AsInnerPtr<sys::Store> for Store {
    unsafe fn as_ptr(&self) -> *mut sys::Store {
        self.inner.as_ptr()
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
    pub fn open(uri: Option<&str>) -> Result<Self, NixideError> {
        let uri_ptr = match uri.map(CString::new) {
            Some(Ok(c_uri)) => c_uri.as_ptr(),
            Some(Err(_)) => Err(new_nixide_error!(StringNulByte))?,
            None => null(),
        };

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
    pub fn realise<F>(
        &self,
        path: &StorePath,
        callback: fn(&str, &StorePath),
    ) -> NixideResult<(String, StorePath)> {
        // // Type alias for our userdata: (outputs vector, context)
        // type Userdata = (
        //     Vec<(String, StorePath)>,
        //     Arc<ErrorContext>,
        //     fn(&str, &StorePath),
        // );
        //
        // // Callback function that will be called for each realized output
        // unsafe extern "C" fn realise_callback(
        //     userdata: *mut c_void,
        //     out_name_ptr: *const c_char,
        //     out_path_ptr: *const sys::StorePath,
        // ) {
        //     // SAFETY: userdata is a valid pointer to our (Vec, Arc<Context>) tuple
        //     let (outputs, ctx, callback) = unsafe { &mut *(userdata as *mut Userdata) };
        //
        //     // SAFETY: outname is a valid C string from Nix
        //     let output_name = if !out_name_ptr.is_null() {
        //         unsafe { CStr::from_ptr(out_name_ptr).to_string_lossy().into_owned() }
        //     } else {
        //         String::from("out") // Default output name
        //     };
        //
        //     // SAFETY: out is a valid StorePath pointer from Nix, we need to clone it
        //     // because Nix owns the original and may free it after the callback
        //     if !out_path_ptr.is_null() {
        //         let cloned_path_ptr =
        //             unsafe { sys::nix_store_path_clone(out_path_ptr as *mut sys::StorePath) };
        //         if let Some(inner) = NonNull::new(cloned_path_ptr) {
        //             let store_path = StorePath { inner };
        //
        //             callback(output_name.as_ref(), &store_path);
        //
        //             outputs.push((output_name, store_path));
        //         }
        //     }
        // }
        //
        // // Create userdata with empty outputs vector and context
        // let mut userdata: Userdata = (Vec::new(), Arc::new(ErrorContext::new()), callback);
        // let userdata_ptr = &mut userdata as *mut Userdata as *mut std::os::raw::c_void;
        //
        // // SAFETY: All pointers are valid, callback is compatible with the FFI signature
        // // - self._context is valid for the duration of this call
        // // - self.inner is valid (checked in Store::open)
        // // - path.inner is valid (checked in StorePath::parse)
        // // - userdata_ptr points to valid stack memory
        // // - realize_callback matches the expected C function signature
        // let err = unsafe {
        //     sys::nix_store_realise(
        //         ctx.as_ptr(),
        //         self.inner.as_ptr(),
        //         path.as_ptr(),
        //         userdata_ptr,
        //         Some(realise_callback),
        //     )
        // };

        wrap::nix_callback!(
            |; userdata: fn(&str, &StorePath);
             output_name_ptr: *const c_char,
             output_path_ptr: *const sys::StorePath|
             -> NixideResult<(String, StorePath)> {
                // XXX: TODO: test to see if this is ever null ("out" as a default feels unsafe...)
                // let output_name = output_name_ptr.to_utf8_string().unwrap_or("out".to_owned());
                // NOTE: this also ensures `output_name_ptr` isn't null
                let output_name = output_name_ptr.to_utf8_string()?;

                let inner = wrap::nix_ptr_fn!(|ctx| unsafe {
                    sys::nix_store_path_clone(output_path_ptr as *mut sys::StorePath)
                })?;
                let store_path = StorePath { inner };

                let callback = unsafe { (*userdata).inner };
                callback(output_name.as_ref(), &store_path);

                Ok((output_name, store_path))
            },
            |callback,
             state: *mut UserData<fn(&str, &StorePath), NixideResult<(String, StorePath)>>,
             ctx: &ErrorContext| unsafe {
                sys::nix_store_realise(
                    ctx.as_ptr(),
                    self.inner.as_ptr(),
                    path.as_ptr(),
                    (*state).inner_ptr() as *mut c_void,
                    Some(callback),
                )
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
    pub fn store_path(&self, path: &str) -> Result<StorePath, NixideError> {
        StorePath::parse(self, path)
    }

    /// Get the version of a Nix store
    ///
    /// If the store doesn't have a version (like the dummy store), returns None
    pub fn version(&self) -> Result<String, NixideError> {
        wrap::nix_string_callback!(
            |callback, userdata: *mut __UserData, ctx: &ErrorContext| unsafe {
                sys::nix_store_get_version(
                    ctx.as_ptr(),
                    self.inner.as_ptr(),
                    Some(callback),
                    userdata as *mut c_void,
                )
            }
        )
    }

    /// Get the URI of a Nix store
    pub fn uri(&self) -> Result<String, NixideError> {
        wrap::nix_string_callback!(
            |callback, userdata: *mut __UserData, ctx: &ErrorContext| unsafe {
                sys::nix_store_get_uri(
                    ctx.as_ptr(),
                    self.inner.as_ptr(),
                    Some(callback),
                    userdata as *mut c_void,
                )
            }
        )
    }

    pub fn store_dir(&self) -> Result<PathBuf, NixideError> {
        wrap::nix_pathbuf_callback!(
            |callback, userdata: *mut __UserData, ctx: &ErrorContext| unsafe {
                sys::nix_store_get_storedir(
                    ctx.as_ptr(),
                    self.inner.as_ptr(),
                    Some(callback),
                    userdata as *mut c_void,
                )
            }
        )
    }

    pub fn copy_closure_to(
        &self,
        dst_store: &Store,
        store_path: &StorePath,
    ) -> Result<(), NixideError> {
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_store_copy_closure(
                ctx.as_ptr(),
                self.as_ptr(),
                dst_store.as_ptr(),
                store_path.as_ptr(),
            ); // semi-colon to return () and not i32
        })
    }

    pub fn copy_closure_from(
        &self,
        src_store: &Store,
        store_path: &StorePath,
    ) -> Result<(), NixideError> {
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_store_copy_closure(
                ctx.as_ptr(),
                src_store.as_ptr(),
                self.as_ptr(),
                store_path.inner.as_ptr(),
            );
        }) // semi-colon to return () and not i32
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        unsafe {
            sys::nix_store_free(self.inner.as_ptr());
        }
    }
}

// SAFETY: Store can be shared between threads
unsafe impl Send for Store {}
unsafe impl Sync for Store {}
