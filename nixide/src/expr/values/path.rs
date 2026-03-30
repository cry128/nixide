use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::path::PathBuf;
use std::ptr::NonNull;

use super::NixValue;
use crate::errors::ErrorContext;
use crate::stdext::CCharPtrExt;
use crate::util::panic_issue_call_failed;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr;
use crate::{EvalState, sys};

pub struct NixPath {
    inner: NonNull<sys::nix_value>,
    state: EvalState,
    value: PathBuf,
}

impl Clone for NixPath {
    fn clone(&self) -> Self {
        let inner = self.inner.clone();

        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_value_incref(ctx.as_ptr(), self.as_ptr());
        })
        .unwrap();

        Self {
            inner,
            state: self.state.clone(),
            value: self.value.clone(),
        }
    }
}

impl Drop for NixPath {
    fn drop(&mut self) {
        let ctx = ErrorContext::new();
        unsafe {
            sys::nix_value_decref(ctx.as_ptr(), self.as_ptr());
        }
    }
}

impl Display for NixPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value.display())
    }
}

impl Debug for NixPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NixPath(\"{}\")", self.value().display())
    }
}

impl AsInnerPtr<sys::nix_value> for NixPath {
    #[inline]
    unsafe fn as_ptr(&self) -> *mut sys::nix_value {
        self.inner.as_ptr()
    }

    #[inline]
    unsafe fn as_ref(&self) -> &sys::nix_value {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    unsafe fn as_mut(&mut self) -> &mut sys::nix_value {
        unsafe { self.inner.as_mut() }
    }
}

impl NixValue for NixPath {
    #[inline]
    fn type_id(&self) -> sys::ValueType {
        sys::ValueType_NIX_TYPE_PATH
    }

    fn from(inner: NonNull<sys::nix_value>, state: &EvalState) -> Self {
        let value = wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_path_string(ctx.as_ptr(), inner.as_ptr())
        })
        .and_then(CCharPtrExt::to_utf8_string)
        .map(PathBuf::from)
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        Self {
            inner,
            state: state.clone(),
            value,
        }
    }
}

impl NixPath {
    /// Returns a shared reference to the underlying value.
    ///
    #[inline]
    pub fn value(&self) -> &PathBuf {
        &self.value
    }

    #[inline]
    pub fn as_path(&self) -> PathBuf {
        self.value.clone()
    }
}
