// XXX: TODO: create wrappers methods to access more than just `info->msg()`
// struct ErrorInfo
// {
//     Verbosity level;
//     HintFmt msg;
//     std::shared_ptr<const Pos> pos;
//     std::list<Trace> traces;
//     /**
//      * Some messages are generated directly by expressions; notably `builtins.warn`, `abort`, `throw`.
//      * These may be rendered differently, so that users can distinguish them.
//      */
//     bool isFromExpr = false;

//     /**
//      * Exit status.
//      */
//     unsigned int status = 1;

//     Suggestions suggestions;

//     static std::optional<std::string> programName;
// };

use std::ffi::{c_char, CStr};
use std::ptr::{null_mut, NonNull};

use crate::error::NixErrorCode;
use crate::sys;
use crate::util::bindings::wrap_libnix_string_callback;

// XXX: TODO: change this to a `Result<T, NixError>`
type NixResult<T> = Result<T, NixErrorCode>;

#[derive(Debug, Clone)]
pub struct NixError {
    pub err: NixErrorCode,
    pub name: String,
    pub msg: String,
    pub info_msg: String,
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
    pub fn peak(&self) -> Option<NixError> {
        // NixError::from( unsafe { sys::nix_err_code(self.as_ptr())}, "");

        // let err = unsafe { sys::nix_err_code(self.inner.as_ptr()) };
        // if err != sys::nix_err_NIX_OK {
        //     // msgp is a borrowed pointer (pointing into the context), so we don't need to free it
        //     let msgp = unsafe { sys::nix_err_msg(null_mut(), self.inner.as_ptr(), null_mut()) };
        //     // Turn the i8 pointer into a Rust string by copying
        //     let msg: &str = unsafe { core::ffi::CStr::from_ptr(msgp).to_str()? };
        //     bail!("{}", msg);
        // }
        // Ok(())

        let result = self.get_code();
        match result {
            Ok(()) => None,
            Err(err) => Some(NixError {
                err,
                name: self.get_name()?,
                msg: self.get_msg()?,
                info_msg: self.get_info_msg()?,
            }),
        }
    }

    pub fn pop(&mut self) -> Option<NixError> {
        let error = self.peak();
        if error.is_some() {
            self.clear();
        }
        error
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

    /// Returns None if no [self.code] is [sys::nix_err_NIX_OK].
    pub(crate) fn get_name(&self) -> Option<String> {
        unsafe {
            let ctx = null_mut();
            // NOTE: an Err here only occurs when "Last error was not a nix error"
            wrap_libnix_string_callback("nix_err_name", |callback, user_data| {
                sys::nix_err_name(ctx, self.as_ptr(), Some(callback), user_data)
            })
            .ok()
        }
    }

    /// Returns None if no [self.code] is [sys::nix_err_NIX_OK].
    /// # Note
    /// On failure [sys::nix_err_name] does the following if the error
    /// has the error code [sys::nix_err_NIX_OK]:
    /// ```c
    /// nix_set_err_msg(context, NIX_ERR_UNKNOWN, "No error message");
    /// return nullptr;
    /// ```
    /// Hence we can just test whether the returned pointer is a `NULL` pointer,
    /// and avoid passing in a [sys::nix_c_context] struct.
    pub(crate) fn get_msg(&self) -> Option<String> {
        unsafe {
            let ctx = null_mut();
            let msg_ptr: *const c_char = sys::nix_err_msg(ctx, self.as_ptr(), null_mut());

            if msg_ptr.is_null() {
                return None;
            }

            match CStr::from_ptr(msg_ptr).to_str() {
                Ok(msg_str) => Some(msg_str.to_string()),
                Err(_) => None,
            }
        }
    }

    /// Returns None if no [self.code] is [sys::nix_err_NIX_OK].
    pub(crate) fn get_info_msg(&self) -> Option<String> {
        unsafe {
            let ctx = null_mut();
            // NOTE: an Err here only occurs when "Last error was not a nix error"
            wrap_libnix_string_callback("nix_err_name", |callback, user_data| {
                sys::nix_err_info_msg(ctx, self.as_ptr(), Some(callback), user_data)
            })
            .ok()
        }
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
