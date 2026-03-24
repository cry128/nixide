use std::fmt::{Display, Formatter, Result as FmtResult};

use super::NixError;
use crate::sys;

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
        crate::NixideError::NixError {
            trace: ::stdext::debug_name!(),
            inner: $inner,
            err: $err,
            msg: $msg,
        }
    }};
    (StringNulByte) => {{
        crate::NixideError::StringNulByte {
            trace: ::stdext::debug_name!(),
        }
    }};
    (StringNotUtf8) => {{
        crate::NixideError::StringNotUtf8 {
            trace: ::stdext::debug_name!(),
        }
    }};
    (NullPtr) => {{
        crate::NixideError::NullPtr {
            trace: ::stdext::debug_name!(),
        }
    }};
    (InvalidArg, $name:expr, $reason:expr) => {{
        crate::NixideError::InvalidArg {
            trace: ::stdext::debug_name!(),
            name: $name,
            reason: $reason,
        }
    }};
    (InvalidType, $expected:expr, $got:expr) => {{
        crate::NixideError::InvalidType {
            trace: ::stdext::debug_name!(),
            expected: $expected,
            got: $got,
        }
    }};
}
pub(crate) use new_nixide_error;

#[allow(unused_macros)]
macro_rules! retrace_nixide_error {
    ($x:expr) => {{
        crate::errors::new_nixide_error!($x.err)
    }};
}
pub(crate) use retrace_nixide_error;

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

pub trait AsErr<T> {
    fn as_err(self) -> Result<(), T>;
}

impl AsErr<NixideError> for Option<NixideError> {
    fn as_err(self) -> Result<(), NixideError> {
        match self {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}
