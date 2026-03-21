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

use std::ffi::c_uint;
use std::ptr::NonNull;

use super::{NixError, NixideError, NixideResult};
use crate::sys;
use crate::util::bindings::wrap_libnix_string_callback;
use crate::util::wrappers::AsInnerPtr;
use crate::util::{panic_issue_call_failed, CCharPtrNixExt};

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
/// # Nix C++ API Internals
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
pub(crate) struct ErrorContext {
    // XXX: TODO: add a RwLock to this (maybe Arc<RwLock>? or is that excessive?)
    inner: NonNull<sys::nix_c_context>,
}

impl AsInnerPtr<sys::nix_c_context> for ErrorContext {
    unsafe fn as_ptr(&self) -> *mut sys::nix_c_context {
        self.inner.as_ptr()
    }
}

impl Into<NixideResult<()>> for &ErrorContext {
    fn into(self) -> NixideResult<()> {
        let inner = self.get_err().ok_?;
        let msg = self.get_msg()?;

        let err = match inner {
            sys::nix_err_NIX_OK => unreachable!(),

            sys::nix_err_NIX_ERR_OVERFLOW => NixError::Overflow,
            sys::nix_err_NIX_ERR_KEY => NixError::KeyNotFound(None),
            sys::nix_err_NIX_ERR_NIX_ERROR => NixError::ExprEval {
                name: self
                    .get_nix_err_name()
                    .unwrap_or_else(|| panic_issue_call_failed!()),

                info_msg: self
                    .get_nix_err_info_msg()
                    .unwrap_or_else(|| panic_issue_call_failed!()),
            },

            sys::nix_err_NIX_ERR_UNKNOWN => NixError::Unknown,
            err => NixError::Undocumented(err),
        };

        Some(new_nixide_error!(NixError, inner, err, msg))
    }
}

impl ErrorContext {
    /// Create a new error context.
    ///
    /// # Errors
    ///
    /// Returns an error if no memory can be allocated for
    /// the underlying [sys::nix_c_context] struct.
    pub fn new() -> Self {
        match NonNull::new(unsafe { sys::nix_c_context_create() }) {
            Some(inner) => ErrorContext { inner },
            None => panic!("[nixide] CRITICAL FAILURE: Out-Of-Memory condition reached - `sys::nix_c_context_create` allocation failed!"),
        }

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
    }

    /// Check the error code and return an error if it's not `NIX_OK`.
    pub fn peak(&self) -> NixideResult<()> {
        match self.into() {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    ///
    /// Equivalent to running `self.peak()` then `self.clear()`
    pub fn pop(&mut self) -> NixideResult<()> {
        let error = self.peak();
        self.clear();
        error
    }

    /// # Nix C++ API Internals
    ///
    /// ```cpp
    /// void nix_clear_err(nix_c_context * context)
    /// {
    ///     if (context)
    ///         context->last_err_code = NIX_OK;
    /// }
    /// ```
    ///
    /// `nix_clear_err` only modifies the `last_err_code`, it does not
    /// clear all attributes of a `nix_c_context` struct. Hence all uses
    /// of `nix_c_context` must be careful to check the `last_err_code` regularly.
    pub fn clear(&mut self) {
        unsafe {
            // NOTE: previous nixops4 used the line: (maybe for compatability with old versions?)
            // sys::nix_set_err_msg(self.inner.as_ptr(), sys::nix_err_NIX_OK, c"".as_ptr());
            sys::nix_clear_err(self.as_ptr());
        }
    }

    /// Returns [None] if [self.code] is [sys::nix_err_NIX_OK], and [Some] otherwise.
    ///
    /// # Nix C++ API Internals
    /// ```cpp
    /// nix_err nix_err_code(const nix_c_context * read_context)
    /// {
    ///     return read_context->last_err_code;
    /// }
    /// ```
    /// This function **never fails**.
    pub(super) fn get_err(&self) -> Option<sys::nix_err> {
        let err = unsafe { sys::nix_err_code(self.as_ptr()) };

        match err {
            sys::nix_err_NIX_OK => None,
            _ => Some(err),
        }
    }

    /// Returns [None] if [self.code] is [sys::nix_err_NIX_OK], and [Some] otherwise.
    ///
    /// # Nix C++ API Internals
    /// ```cpp
    /// const char * nix_err_msg(nix_c_context * context, const nix_c_context * read_context, unsigned int * n)
    /// {
    ///     if (context)
    ///         context->last_err_code = NIX_OK;
    ///     if (read_context->last_err && read_context->last_err_code != NIX_OK) {
    ///         if (n)
    ///             *n = read_context->last_err->size();
    ///         return read_context->last_err->c_str();
    ///     }
    ///     nix_set_err_msg(context, NIX_ERR_UNKNOWN, "No error message");
    ///     return nullptr;
    /// }
    /// ```
    ///
    /// # Note
    /// On failure [sys::nix_err_name] does the following if the error
    /// has the error code [sys::nix_err_NIX_OK]:
    /// ```cpp
    /// nix_set_err_msg(context, NIX_ERR_UNKNOWN, "No error message");
    /// return nullptr;
    /// ```
    /// Hence we can just test whether the returned pointer is a `NULL` pointer,
    /// and avoid passing in a [sys::nix_c_context] struct.
    pub(super) fn get_msg(&self) -> Option<String> {
        // XXX: TODO: what happens if i DO actually use `null_mut` instead of ErrorContext::new? does rust just panic?
        let ctx = ErrorContext::new();
        unsafe {
            // NOTE: an Err here only occurs when `self.get_code() == Ok(())`
            let mut n: c_uint = 0;
            sys::nix_err_msg(ctx.as_ptr(), self.as_ptr(), &mut n)
                .to_utf8_string()
                .ok()
        }
    }

    /// Returns [None] if [self.code] is [sys::nix_err_NIX_OK], and [Some] otherwise.
    ///
    /// # Nix C++ API Internals
    ///
    /// ```cpp
    /// // NOTE(nixide): the implementation of `nix_err_info_msg` is identical to `nix_err_name`
    /// nix_err nix_err_info_msg(
    ///     nix_c_context * context,
    ///     const nix_c_context * read_context,
    ///     nix_get_string_callback callback,
    ///     void * user_data)
    /// {
    ///     if (context)
    ///         context->last_err_code = NIX_OK;
    ///     if (read_context->last_err_code != NIX_ERR_NIX_ERROR) {
    ///         // NOTE(nixide): `nix_set_err_msg` throws a `nix::Error` exception if `context == nullptr`
    ///         return nix_set_err_msg(context, NIX_ERR_UNKNOWN, "Last error was not a nix error");
    ///     }
    ///     // NOTE(nixide): `call_nix_get_string_callback` always returns `NIX_OK`
    ///     return call_nix_get_string_callback(read_context->info->msg.str(), callback, user_data);
    /// }
    /// ```
    ///
    /// `nix_err_info_msg` accepts two `nix_c_context*`:
    /// * `nix_c_context* context` - errors from the function call are logged here
    /// * `const nix_c_context* read_context` - the context to read `info_msg` from
    ///
    /// `nix_set_err_msg` will cause undefined behaviour if `context` is a null pointer (see below)
    /// due to [https://github.com/rust-lang/rust-bindgen/issues/1208].
    /// So we should never assigned it [std::ptr::null_mut].
    /// ```cpp
    /// if (context == nullptr) {
    ///     throw nix::Error("Nix C api error: %s", msg);
    /// }
    /// ```
    pub(super) fn get_nix_err_name(&self) -> Option<String> {
        // XXX: TODO: what happens if i DO actually use `null_mut` instead of ErrorContext::new? does rust just panic?
        unsafe {
            // NOTE: an Err here only occurs when "Last error was not a nix error"
            wrap_libnix_string_callback(|ctx, callback, user_data| {
                sys::nix_err_name(ctx.as_ptr(), self.as_ptr(), Some(callback), user_data)
            })
            .ok()
        }
    }

    /// Returns [None] if [self.code] is [sys::nix_err_NIX_OK], and [Some] otherwise.
    ///
    /// # Nix C++ API Internals
    ///
    /// ```cpp
    /// // NOTE(nixide): the implementation of `nix_err_info_msg` is identical to `nix_err_name`
    /// nix_err nix_err_info_msg(
    ///     nix_c_context * context,
    ///     const nix_c_context * read_context,
    ///     nix_get_string_callback callback,
    ///     void * user_data)
    /// {
    ///     if (context)
    ///         context->last_err_code = NIX_OK;
    ///     if (read_context->last_err_code != NIX_ERR_NIX_ERROR) {
    ///         // NOTE(nixide): `nix_set_err_msg` throws a `nix::Error` exception if `context == nullptr`
    ///         return nix_set_err_msg(context, NIX_ERR_UNKNOWN, "Last error was not a nix error");
    ///     }
    ///     // NOTE(nixide): `call_nix_get_string_callback` always returns `NIX_OK`
    ///     return call_nix_get_string_callback(read_context->info->msg.str(), callback, user_data);
    /// }
    /// ```
    ///
    /// `nix_err_info_msg` accepts two `nix_c_context*`:
    /// * `nix_c_context* context` - errors from the function call are logged here
    /// * `const nix_c_context* read_context` - the context to read `info_msg` from
    ///
    /// `nix_set_err_msg` will cause undefined behaviour if `context` is a null pointer (see below)
    /// due to [https://github.com/rust-lang/rust-bindgen/issues/1208].
    /// So we should never assigned it [std::ptr::null_mut].
    /// ```cpp
    /// if (context == nullptr) {
    ///     throw nix::Error("Nix C api error: %s", msg);
    /// }
    /// ```
    pub(super) fn get_nix_err_info_msg(&self) -> Option<String> {
        // XXX: TODO: what happens if i DO actually use `null_mut` instead of ErrorContext::new? does rust just panic?
        unsafe {
            // NOTE: an Err here only occurs when "Last error was not a nix error"
            wrap_libnix_string_callback(|ctx, callback, user_data| {
                sys::nix_err_info_msg(ctx.as_ptr(), self.as_ptr(), Some(callback), user_data)
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
