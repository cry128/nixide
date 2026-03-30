use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::{self, NonNull};
use std::rc::Rc;

use super::{NixThunk, NixValue, Value};
use crate::errors::{ErrorContext, NixideError};
use crate::stdext::{AsCPtr, CCharPtrExt};
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::util::{panic_issue_call_failed, wrap};
use crate::{EvalState, NixError};

pub struct NixAttrs {
    inner: NonNull<sys::nix_value>,
    state: Rc<RefCell<NonNull<sys::EvalState>>>,
    len: u32,
}

impl Clone for NixAttrs {
    fn clone(&self) -> Self {
        let inner = self.inner.clone();

        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_value_incref(ctx.as_ptr(), self.as_ptr());
        })
        .unwrap();

        Self {
            inner,
            state: self.state.clone(),
            len: self.len,
        }
    }
}

impl Drop for NixAttrs {
    fn drop(&mut self) {
        let ctx = ErrorContext::new();
        unsafe {
            sys::nix_value_decref(ctx.as_ptr(), self.as_ptr());
        }
    }
}

impl Display for NixAttrs {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{{ <attrs> }}")
    }
}

impl Debug for NixAttrs {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NixAttrs({{ <attrs> }})") // XXX: TODO: format attrNames into here
    }
}

impl AsInnerPtr<sys::nix_value> for NixAttrs {
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

impl NixValue for NixAttrs {
    #[inline]
    fn type_id(&self) -> sys::ValueType {
        sys::ValueType_NIX_TYPE_ATTRS
    }

    fn from(inner: NonNull<sys::nix_value>, state: &EvalState) -> Self {
        let len = wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_attrs_size(ctx.as_ptr(), inner.as_ptr())
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        Self {
            inner,
            state: state.inner_ref().clone(),
            len,
        }
    }
}

impl NixAttrs {
    #[inline]
    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn get_idx(&self, index: u32) -> Option<(String, Value)> {
        if index >= self.len {
            return None;
        }

        let name_ptr = ptr::null_mut();

        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_attr_byidx(
                ctx.as_ptr(),
                self.as_ptr(),
                self.state.borrow().as_ptr(),
                index,
                name_ptr,
            )
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        let name = (unsafe { *name_ptr })
            .to_utf8_string()
            .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        let value = Value::from((inner, &self.state));

        Some((name, value))
    }

    pub fn get_idx_lazy(&self, index: u32) -> Option<(String, NixThunk)> {
        if index >= self.len {
            return None;
        }

        let name_ptr = ptr::null_mut();

        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_attr_byidx_lazy(
                ctx.as_ptr(),
                self.as_ptr(),
                self.state.borrow().as_ptr(),
                index,
                name_ptr,
            )
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        let name = (unsafe { *name_ptr })
            .to_utf8_string()
            .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        let value = <NixThunk as NixValue>::from(inner, &self.state);

        Some((name, value))
    }

    pub fn get_name_idx(&self, index: u32) -> Option<String> {
        if index >= self.len {
            return None;
        }

        let name_ptr = wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_attr_name_byidx(
                ctx.as_ptr(),
                self.as_ptr(),
                self.state.borrow().as_ptr(),
                index,
            )
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        let name = name_ptr
            .to_utf8_string()
            .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        Some(name)
    }

    pub fn get<T>(&self, name: T) -> Option<Value>
    where
        T: AsRef<str>,
    {
        let result = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_attr_byname(
                ctx.as_ptr(),
                self.as_ptr(),
                self.state.borrow().as_ptr(),
                name.as_ref()
                    .into_c_ptr()
                    .unwrap_or_else(|err| panic_issue_call_failed!("{}", err)),
            )
        });

        match result {
            Ok(inner) => Some(Value::from((inner, &self.state))),

            Err(NixideError::NixError {
                err: NixError::KeyNotFound(_),
                ..
            }) => None,
            Err(err) => panic_issue_call_failed!("{}", err),
        }
    }

    pub fn get_lazy<T>(&self, name: T) -> Option<NixThunk>
    where
        T: AsRef<str>,
    {
        let result = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_attr_byname_lazy(
                ctx.as_ptr(),
                self.as_ptr(),
                self.state.borrow().as_ptr(),
                name.as_ref()
                    .into_c_ptr()
                    .unwrap_or_else(|err| panic_issue_call_failed!("{}", err)),
            )
        });

        match result {
            Ok(inner) => Some(<NixThunk as NixValue>::from(inner, &self.state)),

            Err(NixideError::NixError {
                err: NixError::KeyNotFound(_),
                ..
            }) => None,
            Err(err) => panic_issue_call_failed!("{}", err),
        }
    }
}
