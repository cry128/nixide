use std::fmt::{Display, Formatter, Result as FmtResult};

use super::{ErrorContext, NixError};
use crate::sys;
use crate::util::panic_issue_call_failed;

pub type NixideResult<T> = Result<T, NixideError>;

#[derive(Debug, Clone)]
pub enum NixideError {
    /// # Warning
    /// [NixideErrorVariant::NixError] is **not the same** as [sys::nix_err_NIX_ERR_NIX_ERROR],
    /// that is instead mapped to [NixError::ExprEval]
    NixError {
        trace: String,
        inner: sys::nix_err,
        err: NixError,
        msg: String,
    },

    /// Returned if a C string `*const c_char` contained a `\0` byte prematurely.
    StringNulByte { trace: String },

    /// Returned if a C string is not encoded in UTF-8.
    StringNotUtf8 { trace: String },

    /// Equivalent to the standard [std::ffi::NulError] type.
    NullPtr { trace: String },

    /// Invalid Argument
    InvalidArg {
        trace: String,
        name: &'static str,
        reason: String,
    },

    /// Invalid Type
    InvalidType {
        trace: String,
        expected: &'static str,
        got: String,
    },
}

macro_rules! new_nixide_error {
    (NixError, $inner:expr, $err:expr, $msg:expr) => {{
        NixideError::NixError {
            trace: stdext::debug_name!(),
            inner: $inner,
            err: $err,
            msg: $msg,
        }
    }};
    (StringNulByte) => {{
        NixideError::StringNulByte {
            trace: stdext::debug_name!(),
        }
    }};
    (StringNotUtf8) => {{
        NixideError::StringNotUtf8 {
            trace: stdext::debug_name!(),
        }
    }};
    (NullPtr) => {{
        NixideError::NullPtr {
            trace: stdext::debug_name!(),
        }
    }};
    (InvalidArg, $name:expr, $reason:expr) => {{
        NixideError::InvalidArg {
            trace: stdext::debug_name!(),
            name: $name,
            reason: $reason,
        }
    }};
    (InvalidType, $expected:expr, $got:expr) => {{
        NixideError::InvalidType {
            trace: stdext::debug_name!(),
            expected: $expected,
            got: $got,
        }
    }};
}
pub(crate) use new_nixide_error;

macro_rules! retrace_nixide_error {
    ($x:expr) => {{
        new_nixide_error!($x.err)
    }};
}
pub(crate) use retrace_nixide_error;

impl NixideError {
    /// # Panics
    ///
    /// This function will panic in the event that `context.get_err() == Some(err) && err == sys::nix_err_NIX_OK`
    /// since `nixide::ErrorContext::get_err` is expected to return `None` to indicate `sys::ni_err_NIX_OK`.
    ///
    ///
    /// This function will panic in the event that `value != sys::nix_err_NIX_OK`
    /// but that `context.get_code() == sys::nix_err_NIX_OK`
    pub(super) fn from_error_context(context: &ErrorContext) -> Option<NixideError> {
        let inner = context.get_err()?;
        let msg = context.get_msg()?;

        #[allow(nonstandard_style)]
        let err = match inner {
            sys::nix_err_NIX_OK => unreachable!(),

            sys::nix_err_NIX_ERR_OVERFLOW => NixError::Overflow,
            sys::nix_err_NIX_ERR_KEY => NixError::KeyNotFound(None),
            sys::nix_err_NIX_ERR_NIX_ERROR => NixError::ExprEval {
                name: context
                    .get_nix_err_name()
                    .unwrap_or_else(|| panic_issue_call_failed!()),

                info_msg: context
                    .get_nix_err_info_msg()
                    .unwrap_or_else(|| panic_issue_call_failed!()),
            },

            sys::nix_err_NIX_ERR_UNKNOWN => NixError::Unknown,
            err => NixError::Undocumented(err),
        };

        Some(new_nixide_error!(NixError, inner, err, msg))
    }
}

impl std::error::Error for NixideError {}

impl Display for NixideError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            NixideError::NixError {
                trace,
                inner,
                err,
                msg,
            } => {
                write!(f, "[nixide ~ {trace}]{err} (nix_err={inner}): {msg}")
            }

            NixideError::StringNulByte { trace } => {
                write!(f, "[nixide ~ {trace}] Got premature `\\0` (NUL) byte")
            }

            NixideError::StringNotUtf8 { trace } => {
                write!(f, "[nixide ~ {trace}] Expected UTF-8 encoded string")
            }

            NixideError::NullPtr { trace } => write!(f, "[nixide ~ {trace}] Got null pointer"),

            NixideError::InvalidArg {
                trace,
                name,
                reason,
            } => {
                write!(
                    f,
                    "[nixide ~ {trace}] Invalid argument `{name}`: reason \"{reason}\""
                )
            }

            NixideError::InvalidType {
                trace,
                expected,
                got,
            } => write!(
                f,
                "[nixide ~ {trace}] Got invalid type: expected `{expected}` but got `{got}`"
            ),
        }
    }
}
