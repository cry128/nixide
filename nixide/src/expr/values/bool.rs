use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;
use std::rc::Rc;

use super::NixValue;
use crate::errors::ErrorContext;
use crate::util::panic_issue_call_failed;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr;
use crate::{EvalState, sys};

pub struct NixBool {
    inner: NonNull<sys::nix_value>,
    state: Rc<RefCell<NonNull<sys::EvalState>>>,
    value: bool,
}

impl Clone for NixBool {
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

impl Drop for NixBool {
    fn drop(&mut self) {
        let ctx = ErrorContext::new();
        unsafe {
            sys::nix_value_decref(ctx.as_ptr(), self.as_ptr());
        }
    }
}

impl Display for NixBool {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value())
    }
}

impl Debug for NixBool {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NixBool({})", self.value)
    }
}

impl AsInnerPtr<sys::nix_value> for NixBool {
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

impl NixValue for NixBool {
    #[inline]
    fn type_id(&self) -> sys::ValueType {
        sys::ValueType_NIX_TYPE_BOOL
    }

    fn from(inner: NonNull<sys::nix_value>, state: &EvalState) -> Self {
        let value = wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_bool(ctx.as_ptr(), inner.as_ptr())
        })
        .unwrap_or_else(|err| {
            panic_issue_call_failed!("`sys::nix_get_bool` failed for valid `NixBool` ({})", err)
        });

        Self {
            inner,
            state: state.inner_ref().clone(),
            value,
        }
    }
}

impl NixBool {
    /// Returns a shared reference to the underlying value.
    ///
    #[inline]
    pub fn value(&self) -> &bool {
        &self.value
    }

    #[inline]
    pub fn as_bool(&self) -> bool {
        self.value
    }
}
