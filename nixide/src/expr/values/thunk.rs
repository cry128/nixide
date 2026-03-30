use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;
use std::rc::Rc;

use super::{NixValue, Value};
use crate::EvalState;
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::util::{panic_issue_call_failed, wrap};

pub struct NixThunk {
    inner: NonNull<sys::nix_value>,
    state: Rc<RefCell<NonNull<sys::EvalState>>>,
}

impl Clone for NixThunk {
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

impl Drop for NixThunk {
    fn drop(&mut self) {
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_value_decref(ctx.as_ptr(), self.as_ptr());
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err))
    }
}

impl Display for NixThunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "<thunk>")
    }
}

impl Debug for NixThunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NixThunk")
    }
}

impl AsInnerPtr<sys::nix_value> for NixThunk {
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

impl NixValue for NixThunk {
    #[inline]
    fn type_id(&self) -> sys::ValueType {
        sys::ValueType_NIX_TYPE_THUNK
    }

    fn from(inner: NonNull<sys::nix_value>, state: &EvalState) -> Self {
        Self {
            inner,
            state: state.inner_ref().clone(),
        }
    }
}

impl NixThunk {
    pub fn eval(self) -> Value {
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_value_force(
                ctx.as_ptr(),
                self.state.borrow().as_ptr(),
                self.inner.as_ptr(),
            )
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        Value::from((self.inner, &self.state))
    }
}
