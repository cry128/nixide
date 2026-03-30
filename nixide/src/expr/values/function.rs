use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;
use std::rc::Rc;

use super::{NixValue, Value};
use crate::errors::ErrorContext;
use crate::stdext::SliceExt;
use crate::util::wrappers::AsInnerPtr;
use crate::util::{panic_issue_call_failed, wrap};
use crate::{EvalState, sys};

pub struct NixFunction {
    inner: NonNull<sys::nix_value>,
    state: Rc<RefCell<NonNull<sys::EvalState>>>,
}

impl Clone for NixFunction {
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

impl Drop for NixFunction {
    fn drop(&mut self) {
        let ctx = ErrorContext::new();
        unsafe {
            sys::nix_value_decref(ctx.as_ptr(), self.as_ptr());
        }
    }
}

impl Display for NixFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "<function>")
    }
}

impl Debug for NixFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NixFunction")
    }
}

impl AsInnerPtr<sys::nix_value> for NixFunction {
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

impl NixValue for NixFunction {
    #[inline]
    fn type_id(&self) -> sys::ValueType {
        sys::ValueType_NIX_TYPE_FUNCTION
    }

    fn from(inner: NonNull<sys::nix_value>, state: &EvalState) -> Self {
        Self {
            inner,
            state: state.inner_ref().clone(),
        }
    }
}

impl NixFunction {
    pub fn call<T>(&self, arg: &T) -> Value
    where
        T: NixValue,
    {
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_alloc_value(ctx.as_ptr(), self.state.borrow().as_ptr())
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_value_call(
                ctx.as_ptr(),
                self.state.borrow().as_ptr(),
                self.as_ptr(),
                arg.as_ptr(),
                inner.as_ptr(),
            );
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        Value::from((inner, &self.state))
    }

    pub fn call_many<T>(&self, args: &[&T]) -> Value
    where
        T: NixValue,
    {
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_alloc_value(ctx.as_ptr(), self.state.borrow().as_ptr())
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_value_call_multi(
                ctx.as_ptr(),
                self.state.borrow().as_ptr(),
                self.as_ptr(),
                args.len(),
                args.into_c_array(),
                inner.as_ptr(),
            );
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        Value::from((inner, &self.state))
    }
}

// #[cfg(nightly)]
// impl<T> Fn<(&T,)> for NixFunction
// where
//     T: NixValue,
// {
//     extern "rust-call" fn call(&self, args: (&T,)) -> Value {
//         self.call(args.0)
//     }
// }
