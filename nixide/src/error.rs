use std::ffi::NulError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;

use crate::sys;

/// Standard (nix_err) and some additional error codes
/// produced by the libnix C API.
#[derive(Debug)]
pub enum NixError {
    /// A generic Nix error occurred.
    ///
    /// # Reason
    ///
    /// This error code is returned when a generic Nix error occurred during the
    /// function execution.
    NixError { location: &'static str },

    /// An overflow error occurred.
    ///
    /// # Reason
    ///
    /// This error code is returned when an overflow error occurred during the
    /// function execution.
    Overflow { location: &'static str },

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
    KeyNotFound {
        location: &'static str,
        key: Option<String>,
    },

    /// An unknown error occurred.
    ///
    /// # Reason
    ///
    /// This error code is returned when an unknown error occurred during the
    /// function execution.
    Unknown {
        location: &'static str,
        reason: String,
    },

    /// An undocumented error occurred.
    ///
    /// # Reason
    ///
    /// The libnix C API defines `enum nix_err` as a signed integer value.
    /// In the (unexpected) event libnix returns an error code with an
    /// invalid enum value, or one I new addition I didn't know existed,
    /// then an [NixError::Undocumented] is considered to have occurred.
    Undocumented {
        location: &'static str,
        err_code: sys::nix_err,
    },

    //////////////////////
    // NON-STANDARD ERRORS
    //////////////////////
    /// NulError
    NulError { location: &'static str },

    /// Non-standard
    NullPtr { location: &'static str },

    /// Invalid Argument
    InvalidArg {
        location: &'static str,
        reason: &'static str, // XXX: TODO: make this a String
    },

    /// Invalid Type
    InvalidType {
        location: &'static str,
        expected: &'static str,
        got: String,
    },
}

impl NixError {
    pub fn from(err_code: sys::nix_err, location: &'static str) -> Result<(), NixError> {
        #[allow(nonstandard_style)]
        match err_code {
            sys::nix_err_NIX_OK => Ok(()),

            sys::nix_err_NIX_ERR_OVERFLOW => Err(NixError::Overflow { location }),
            sys::nix_err_NIX_ERR_KEY => Err(NixError::KeyNotFound {
                location,
                key: None,
            }),
            sys::nix_err_NIX_ERR_NIX_ERROR => Err(NixError::NixError { location }),

            sys::nix_err_NIX_ERR_UNKNOWN => Err(NixError::Unknown {
                location,
                reason: "Unknown error occurred".to_string(),
            }),
            _ => Err(NixError::Undocumented { location, err_code }),
        }
    }

    pub fn from_nulerror<T>(
        result: Result<T, NulError>,
        location: &'static str,
    ) -> Result<T, Self> {
        result.or(Err(NixError::NulError { location }))
    }

    pub fn new_nonnull<T>(ptr: *mut T, location: &'static str) -> Result<NonNull<T>, Self>
    where
        T: Sized,
    {
        NonNull::new(ptr).ok_or(NixError::NullPtr { location })
    }
}

impl std::error::Error for NixError {}

impl Display for NixError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let msg = match self {
            NixError::NixError { location } => {
                format!("[libnix] Generic error (at location `{location}`)")
            }
            NixError::Overflow { location } => {
                format!("[libnix] Overflow error (at location `{location}`)")
            }
            NixError::KeyNotFound { location, key } => format!(
                "[libnix] Key not found {} (at location `{location}`)",
                match key {
                    Some(key) => format!("`{key}`"),
                    None => "".to_owned(),
                }
            ),

            NixError::Unknown { location, reason } => {
                format!("Unknown error \"{reason}\" (at location `{location}`)")
            }
            NixError::Undocumented { location, err_code } => {
                format!(
                    "[libnix] An undocumented nix_err was returned with {err_code} (at location `{location}`)"
                )
            }

            NixError::NulError { location } => {
                format!("Nul error (at location `{location}`)")
            }
            NixError::NullPtr { location } => {
                format!("[libnix] Null pointer (at location `{location}`)")
            }

            NixError::InvalidArg { location, reason } => {
                format!("Invalid argument \"{reason}\" (at location `{location}`)")
            }
            NixError::InvalidType {
                location,
                expected,
                got,
            } => {
                format!("Invalid type, expected \"{expected}\" ${got} (at location `{location}`)")
            }
        };

        write!(f, "{msg}")
    }
}
