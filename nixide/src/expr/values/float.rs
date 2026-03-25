use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;

use super::NixValue;
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::util::{panic_issue_call_failed, wrap};

pub struct NixFloat {
    inner: NonNull<sys::nix_value>,
    value: f64,
}

impl Display for NixFloat {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "<float>")
    }
}

impl Debug for NixFloat {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NixFloat(${})", self.value())
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
    fn get_enum_id(&self) -> sys::ValueType {
        sys::ValueType_NIX_TYPE_FLOAT
    }

    fn new(inner: NonNull<sys::nix_value>) -> Self {
        let value = wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_float(ctx.as_ptr(), inner.as_ptr())
        })
        .unwrap_or_else(|err| {
            panic_issue_call_failed!("`sys::nix_get_float` failed for valid `NixFloat` ({})", err)
        });

        Self { inner, value }
    }
}

impl NixFloat {
    /// Returns a shared reference to the underlying value.
    ///
    #[inline]
    fn value(&self) -> &f64 {
        &self.value
    }
}
