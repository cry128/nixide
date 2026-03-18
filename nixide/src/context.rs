use std::ffi::{c_char, c_uint, CStr, CString};
use std::ptr::{null_mut, NonNull};

use crate::error::NixErrorCode;
use crate::sys;
use crate::util::bindings::wrap_libnix_string_callback;

// XXX: TODO: change this to a `Result<T, NixError>`
type NixResult<T> = Result<T, NixErrorCode>;

pub struct NixError {
    pub code: NixErrorCode,
    pub name: String,
    pub msg: Option<String>,
    pub info_msg: Option<String>,
}

/// This object stores error state.
///
/// Passed as a first parameter to functions that can fail, to store error
/// information.
///
/// # Warning
///
/// These can be reused between different function calls,
/// but make sure not to use them for multiple calls simultaneously
/// (which can happen in callbacks).
///
/// # `libnix` API Internals
///
/// ```cpp
/// struct nix_c_context
/// {
///     nix_err last_err_code = NIX_OK;
///     /* WARNING: The last error message. Always check last_err_code.
///        WARNING: This may not have been cleared, so that clearing is fast. */
///     std::optional<std::string> last_err = {};
///     std::optional<nix::ErrorInfo> info = {};
///     std::string name = "";
/// };
/// ```
///
/// The [sys::nix_c_context] struct is laid out so that it can also be
/// cast to a [sys::nix_err] to inspect directly:
/// ```c
/// assert(*(nix_err*)ctx == NIX_OK);
/// ```
///
pub struct ErrorContext {
    inner: NonNull<sys::nix_c_context>,
}

impl ErrorContext {
    /// Create a new Nix context.
    ///
    /// This initializes the Nix C API context and the required libraries.
    ///
    /// # Errors
    ///
    /// Returns an error if context creation or library initialization fails.
    pub fn new() -> Result<Self, NixErrorCode> {
        // SAFETY: nix_c_context_create is safe to call
        let ctx_ptr = unsafe { sys::nix_c_context_create() };
        let ctx = ErrorContext {
            inner: NonNull::new(ctx_ptr).ok_or(NixErrorCode::NullPtr {
                location: "nix_c_context_create",
            })?,
        };

        // Initialize required libraries
        // XXX: TODO: move this to a separate init function (maybe a Nix::init() function)
        // unsafe {
        //     NixErrorCode::from(
        //         sys::nix_libutil_init(ctx.inner.as_ptr()),
        //         "nix_libutil_init",
        //     )?;
        //     NixErrorCode::from(
        //         sys::nix_libstore_init(ctx.inner.as_ptr()),
        //         "nix_libstore_init",
        //     )?;
        //     NixErrorCode::from(
        //         sys::nix_libexpr_init(ctx.inner.as_ptr()),
        //         "nix_libexpr_init",
        //     )?;
        // };

        Ok(ctx)
    }

    /// Get the raw context pointer.
    ///
    /// # Safety
    ///
    /// Although this function isn't inherently `unsafe`, it is
    /// marked as such intentionally to force calls to be wrapped
    /// in `unsafe` blocks for clarity.
    pub(crate) unsafe fn as_ptr(&self) -> *mut sys::nix_c_context {
        self.inner.as_ptr()
    }

    /// Check the error code and return an error if it's not `NIX_OK`.
    ///
    /// We recommend to use `check_call!` if possible.
    pub fn peak(&self) -> Result<(), NixErrorCode> {
        // NixError::from( unsafe { sys::nix_err_code(self.as_ptr())}, "");

        let err = unsafe { sys::nix_err_code(self.inner.as_ptr()) };
        if err != sys::nix_err_NIX_OK {
            // msgp is a borrowed pointer (pointing into the context), so we don't need to free it
            let msgp = unsafe { sys::nix_err_msg(null_mut(), self.inner.as_ptr(), null_mut()) };
            // Turn the i8 pointer into a Rust string by copying
            let msg: &str = unsafe { core::ffi::CStr::from_ptr(msgp).to_str()? };
            bail!("{}", msg);
        }
        Ok(())
    }

    pub fn pop(&mut self) -> Result<(), NixErrorCode> {
        let result = self.peak();
        if result.is_err() {
            self.clear();
        }
        result
    }

    pub fn clear(&mut self) {
        unsafe {
            // NOTE: previous nixops4 used the line: (maybe for compatability with old versions?)
            // sys::nix_set_err_msg(self.inner.as_ptr(), sys::nix_err_NIX_OK, c"".as_ptr());
            sys::nix_clear_err(self.as_ptr());
        }
    }

    ///
    /// Never fails
    pub(crate) fn get_code(&self) -> Result<(), NixErrorCode> {
        NixErrorCode::from(unsafe { sys::nix_err_code(self.as_ptr()) }, "nix_err_code")
    }

    pub(crate) fn get_name(&self, result: NixResult<()>) -> Option<String> {
        match result {
            Err(code) => unsafe {
                let ctx = null_mut();
                wrap_libnix_string_callback("nix_err_name", |callback, user_data| {
                    sys::nix_err_name(ctx, self.as_ptr(), Some(callback), user_data)
                })
            },
            Ok(_) => None,
        }
    }

    /// # Note
    /// On failure [sys::nix_err_name] does the following if the error
    /// has the error code [sys::nix_err_NIX_OK]:
    /// ```c
    /// nix_set_err_msg(context, NIX_ERR_UNKNOWN, "No error message");
    /// return nullptr;
    /// ```
    /// Hence we can just test whether the returned pointer is a `NULL` pointer,
    /// and avoid passing in a [sys::nix_c_context] struct.
    pub(crate) fn get_msg(&self, result: NixResult<()>) -> Option<String> {
        match result {
            Err(_) => unsafe {
                let ctx = null_mut();
                let msg_ptr: *const c_char = sys::nix_err_msg(ctx, self.as_ptr(), null_mut());

                if msg_ptr.is_null() {
                    return None;
                }

                match CStr::from_ptr(msg_ptr).to_str() {
                    Ok(msg_str) => Some(msg_str.to_string()),
                    Err(_) => None,
                }
            },
            Ok(_) => None,
        }
    }

    pub(crate) fn get_info_msg(&self) -> Option<String> {}

    pub fn check_one_call_or_key_none<T, F>(&mut self, f: F) -> Result<Option<T>, NixErrorCode>
    where
        F: FnOnce(*mut sys::nix_c_context) -> T,
    {
        let t = f(unsafe { self.as_ptr() });
        if unsafe { sys::nix_err_code(self.inner.as_ptr()) == sys::nix_err_NIX_ERR_KEY } {
            self.clear();
            return Ok(None);
        }
        self.pop()?;
        Ok(Some(t))
    }
}

impl Drop for ErrorContext {
    fn drop(&mut self) {
        // SAFETY: We own the context and it's valid until drop
        unsafe {
            sys::nix_c_context_free(self.inner.as_ptr());
        }
    }
}
