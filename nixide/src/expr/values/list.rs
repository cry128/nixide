use std::ffi::c_void;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;

use super::{NixThunk, NixValue, Value};
use crate::EvalState;
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::util::{panic_issue_call_failed, wrap};

pub struct NixList {
    inner: NonNull<sys::nix_value>,
    state: EvalState,
}

impl Clone for NixList {
    fn clone(&self) -> Self {
        let inner = self.inner.clone();

        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_gc_incref(ctx.as_ptr(), self.as_ptr() as *mut c_void);
        })
        .unwrap();

        Self {
            inner,
            state: self.state.clone(),
        }
    }
}

impl Drop for NixList {
    fn drop(&mut self) {
        let ctx = ErrorContext::new();
        unsafe {
            sys::nix_value_decref(ctx.as_ptr(), self.as_ptr());
        }
    }
}

impl Display for NixList {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[ <list> ]")
    }
}

impl Debug for NixList {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NixList([ <list> ])")
    }
}

impl AsInnerPtr<sys::nix_value> for NixList {
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

impl NixValue for NixList {
    #[inline]
    fn type_id(&self) -> sys::ValueType {
        sys::ValueType_NIX_TYPE_LIST
    }

    fn from(inner: NonNull<sys::nix_value>, state: &EvalState) -> Self {
        Self {
            inner,
            state: state.clone(),
        }
    }
}

impl NixList {
    /// Forces the evaluation on all elements of the list.
    ///
    pub fn as_vec(&self) -> Vec<Value> {
        // XXX: TODO: should I just return a LazyArray instead?
        let mut value = Vec::new();
        for i in 0..self.len() {
            value.push(self.get(i));
        }

        value
    }

    pub fn as_vec_lazy(&self) -> Vec<NixThunk> {
        // XXX: TODO: should I just return a LazyArray instead?
        let mut value = Vec::new();
        for i in 0..self.len() {
            value.push(self.get_lazy(i));
        }

        value
    }

    /// Get the length of a list. This function preserves
    /// laziness and does not evaluate the internal fields.
    ///
    pub fn len(&self) -> u32 {
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_list_size(ctx.as_ptr(), self.as_ptr())
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err))
    }

    pub fn get(&self, index: u32) -> Value {
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_list_byidx(ctx.as_ptr(), self.as_ptr(), self.state.as_ptr(), index)
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        Value::from((inner, &self.state))
    }

    pub fn get_lazy(&self, index: u32) -> NixThunk {
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_list_byidx_lazy(ctx.as_ptr(), self.as_ptr(), self.state.as_ptr(), index)
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        <NixThunk as NixValue>::from(inner, &self.state)
    }
}
