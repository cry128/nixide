use std::cell::RefCell;
use std::ffi::c_void;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;
use std::rc::Rc;

use super::NixValue;
use crate::errors::ErrorContext;
use crate::util::panic_issue_call_failed;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr;
use crate::{EvalState, sys};

pub struct NixString {
    inner: NonNull<sys::nix_value>,
    state: Rc<RefCell<EvalState>>,
    value: String,
}

impl Drop for NixString {
    fn drop(&mut self) {
        let ctx = ErrorContext::new();
        unsafe {
            sys::nix_value_decref(ctx.as_ptr(), self.as_ptr());
        }
    }
}

impl Display for NixString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "<string>")
    }
}

impl Debug for NixString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NixString(\"${}\")", self.value())
    }
}

impl AsInnerPtr<sys::nix_value> for NixString {
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

impl NixValue for NixString {
    #[inline]
    fn type_id(&self) -> sys::ValueType {
        sys::ValueType_NIX_TYPE_STRING
    }

    fn from(inner: NonNull<sys::nix_value>, state: Rc<RefCell<EvalState>>) -> Self {
        let value = wrap::nix_string_callback!(
            |callback, userdata: *mut __UserData, ctx: &ErrorContext| unsafe {
                sys::nix_get_string(
                    ctx.as_ptr(),
                    inner.as_ptr(),
                    Some(callback),
                    userdata as *mut c_void,
                );
            }
        )
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        Self {
            inner,
            state,
            value,
        }
    }
}

impl NixString {
    /// Returns a shared reference to the underlying value.
    ///
    #[inline]
    fn value(&self) -> &String {
        &self.value
    }
}
