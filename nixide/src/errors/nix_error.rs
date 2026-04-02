use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::sys;

/// Standard (nix_err) and some additional error codes
/// produced by the libnix C API.
#[derive(Debug, Clone)]
pub enum NixError {
    /// A generic Nix error occurred.
    ///
    /// # Reason
    ///
    /// This error code is returned when a generic Nix error occurs
    /// during nixexpr evaluation.
    ///
    /// # Nix C++ API Internals
    ///
    /// ```cpp
    /// // `NIX_ERR_NIX_ERROR` variant of the `nix_err` enum type
    /// NIX_ERR_NIX_ERROR = -4
    /// ```
    ExprEval { name: String, info_msg: String },

    /// A key/index access error occurred in C API functions.
    ///
    /// # Reason
    ///
    /// This error code is returned when accessing a key, index, or identifier that
    /// does not exist in C API functions. Common scenarios include:
    /// - Setting keys that don't exist (nix_setting_get, nix_setting_set)
    /// - List indices that are out of bounds (nix_get_list_byidx*)
    /// - Attribute names that don't exist (nix_get_attr_byname*)
    /// - Attribute indices that are out of bounds (nix_get_attr_byidx*, nix_get_attr_name_byidx)
    ///
    /// This error typically indicates incorrect usage or assumptions about data structure
    /// contents, rather than internal Nix evaluation errors.
    ///
    /// # Note
    ///
    /// This error code should ONLY be returned by C API functions themselves,
    /// not by underlying Nix evaluation. For example, evaluating `{}.foo` in Nix
    /// will throw a normal error (NIX_ERR_NIX_ERROR), not NIX_ERR_KEY.
    ///
    /// # Nix C++ API Internals
    ///
    /// ```cpp
    /// // `NIX_ERR_KEY` variant of the `nix_err` enum type
    /// NIX_ERR_KEY = -3
    /// ```
    KeyNotFound(Option<String>),

    /// An overflow error occurred.
    ///
    /// # Reason
    ///
    /// This error code is returned when an overflow error occurred during the
    /// function execution.
    ///
    /// # Nix C++ API Internals
    ///
    /// ```cpp
    /// // `NIX_ERR_OVERFLOW` variant of the `nix_err` enum type
    /// NIX_ERR_OVERFLOW = -2
    /// ```
    Overflow,

    /// An unknown error occurred.
    ///
    /// # Reason
    ///
    /// This error code is returned when an unknown error occurred during the
    /// function execution.
    ///
    /// # Nix C++ API Internals
    ///
    /// ```cpp
    /// // `NIX_ERR_OVERFLOW` variant of the `nix_err` enum type
    /// NIX_ERR_UNKNOWN = -1
    /// ```
    Unknown,

    ///
    /// An undocumented error occurred.
    ///
    /// # Reason
    ///
    /// The libnix C API defines `enum nix_err` as a signed integer value.
    /// In the (unexpected) event libnix returns an error code with an
    /// invalid enum value, or one I new addition I didn't know existed,
    /// then an [NixError::Undocumented] is considered to have occurred.
    ///
    /// # Nix C++ API Internals
    ///
    /// [NixError::Undocumented] has no equivalent in the `libnix` api.
    /// This is solely a language difference between C++ and Rust, since
    /// [sys::NixErr] is defined over the *"continuous" (not realy)*
    /// type [::core::ffi::c_int].
    Undocumented(sys::NixErr),
}

impl Display for NixError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            NixError::ExprEval { name, info_msg } => write!(
                f,
                "[libnix] NixExpr evaluation failed [name=\"{name}\", info_msg=\"{info_msg}\"]"
            ),
            NixError::KeyNotFound(Some(key)) => write!(f, "[libnix] Key not found \"{key}\""),
            NixError::KeyNotFound(None) => write!(f, "[libnix] Key not found"),
            NixError::Overflow => write!(f, "[libnix] Overflow error"),
            NixError::Unknown => write!(f, "[libnix] Unknown error"),
            NixError::Undocumented(err) => write!(
                f,
                "[libnix] An undocumented nix_err({err}) [please open an issue on https://github.com/cry128/nixide]"
            ),
        }
    }
}

impl NixError {
    pub fn err_code(&self) -> sys::NixErr {
        match self {
            NixError::Overflow => sys::NixErr::Overflow,
            NixError::KeyNotFound(_) => sys::NixErr::Key,
            NixError::ExprEval {
                name: _,
                info_msg: _,
            } => sys::NixErr::NixError,
            NixError::Unknown => sys::NixErr::NixError,
            NixError::Undocumented(err) => err.clone(),
        }
    }
}
