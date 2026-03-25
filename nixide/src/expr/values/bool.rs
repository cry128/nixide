use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;

use super::NixValue;
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::panic_issue_call_failed;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr;

pub struct NixBool {
    inner: NonNull<sys::nix_value>,
    value: bool,
}

impl Display for NixBool {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "<bool>")
    }
}

impl Debug for NixBool {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NixBool(${})", self.value)
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
    fn get_enum_id(&self) -> sys::ValueType {
        sys::ValueType_NIX_TYPE_BOOL
    }

    fn new(inner: NonNull<sys::nix_value>) -> Self {
        let value = wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_bool(ctx.as_ptr(), inner.as_ptr())
        })
        .unwrap_or_else(|err| {
            panic_issue_call_failed!("`sys::nix_get_bool` failed for valid `NixBool` ({})", err)
        });

        Self { inner, value }
    }
}

impl NixBool {
    /// Returns a shared reference to the underlying value.
    ///
    #[inline]
    fn value(&self) -> &bool {
        &self.value
    }
}
