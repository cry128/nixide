use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;
use std::rc::Rc;

use super::NixValue;
use crate::EvalState;
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr;

pub struct NixNull {
    inner: NonNull<sys::nix_value>,
    state: Rc<RefCell<NonNull<sys::EvalState>>>,
}

impl Clone for NixNull {
    fn clone(&self) -> Self {
        let inner = self.inner.clone();

        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_value_incref(ctx.as_ptr(), self.as_ptr());
        })
        .unwrap();

        Self {
            inner,
            state: self.state.clone(),
        }
    }
}

impl Drop for NixNull {
    fn drop(&mut self) {
        let ctx = ErrorContext::new();
        unsafe {
            sys::nix_value_decref(ctx.as_ptr(), self.as_ptr());
        }
    }
}

impl Display for NixNull {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "null")
    }
}

impl Debug for NixNull {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NixNull")
    }
}

impl AsInnerPtr<sys::nix_value> for NixNull {
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

impl NixValue for NixNull {
    #[inline]
    fn type_id(&self) -> sys::ValueType {
        sys::ValueType_NIX_TYPE_NULL
    }

    fn from(inner: NonNull<sys::nix_value>, state: &EvalState) -> Self {
        Self {
            inner,
            state: state.inner_ref().clone(),
        }
    }
}
