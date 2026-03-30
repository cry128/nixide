use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;

use super::NixValue;
use crate::EvalState;
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::util::{panic_issue_call_failed, wrap};

pub struct NixFloat {
    inner: NonNull<sys::nix_value>,
    state: EvalState,
    value: f64,
}

impl Clone for NixFloat {
    fn clone(&self) -> Self {
        let inner = self.inner.clone();

        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_value_incref(ctx.as_ptr(), self.as_ptr());
        })
        .unwrap();

        Self {
            inner,
            state: self.state.clone(),
            value: self.value,
        }
    }
}

impl Drop for NixFloat {
    fn drop(&mut self) {
        let ctx = ErrorContext::new();
        unsafe {
            sys::nix_value_decref(ctx.as_ptr(), self.as_ptr());
        }
    }
}

impl Display for NixFloat {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value())
    }
}

impl Debug for NixFloat {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NixFloat({})", self.value())
    }
}

impl AsInnerPtr<sys::nix_value> for NixFloat {
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

impl NixValue for NixFloat {
    #[inline]
    fn type_id(&self) -> sys::ValueType {
        sys::ValueType_NIX_TYPE_FLOAT
    }

    fn from(inner: NonNull<sys::nix_value>, state: &EvalState) -> Self {
        let value = wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_float(ctx.as_ptr(), inner.as_ptr())
        })
        .unwrap_or_else(|err| {
            panic_issue_call_failed!("`sys::nix_get_float` failed for valid `NixFloat` ({})", err)
        });

        Self {
            inner,
            state: state.clone(),
            value,
        }
    }
}

impl NixFloat {
    /// Returns a shared reference to the underlying value.
    ///
    #[inline]
    pub fn value(&self) -> &f64 {
        &self.value
    }

    #[inline]
    pub fn as_float(&self) -> f64 {
        self.value
    }
}
