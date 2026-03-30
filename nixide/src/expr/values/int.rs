use std::ffi::c_void;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;

use super::NixValue;
use crate::EvalState;
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::util::{panic_issue_call_failed, wrap};

pub struct NixInt {
    inner: NonNull<sys::nix_value>,
    state: EvalState,
    value: i64,
}

impl Clone for NixInt {
    fn clone(&self) -> Self {
        let inner = self.inner.clone();

        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_gc_incref(ctx.as_ptr(), self.as_ptr() as *mut c_void);
        })
        .unwrap();

        Self {
            inner,
            state: self.state.clone(),
            value: self.value,
        }
    }
}

impl Drop for NixInt {
    fn drop(&mut self) {
        let ctx = ErrorContext::new();
        unsafe {
            sys::nix_value_decref(ctx.as_ptr(), self.as_ptr());
        }
    }
}

impl Display for NixInt {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value())
    }
}

impl Debug for NixInt {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NixInt({})", self.value())
    }
}

impl AsInnerPtr<sys::nix_value> for NixInt {
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

impl NixValue for NixInt {
    #[inline]
    fn type_id(&self) -> sys::ValueType {
        sys::ValueType_NIX_TYPE_INT
    }

    fn from(inner: NonNull<sys::nix_value>, state: &EvalState) -> Self {
        let value = wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_int(ctx.as_ptr(), inner.as_ptr())
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        Self {
            inner,
            state: state.clone(),
            value,
        }
    }
}

impl NixInt {
    /// Returns a shared reference to the underlying value.
    ///
    #[inline]
    pub fn value(&self) -> &i64 {
        &self.value
    }

    #[inline]
    pub fn as_int(&self) -> i64 {
        self.value
    }
}
