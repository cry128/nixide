use std::cell::LazyCell;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;

use super::NixValue;
use crate::expr::RealisedString;
use crate::util::panic_issue_call_failed;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr;
use crate::{sys, NixideResult};

pub struct NixString {
    inner: NonNull<sys::nix_value>,
    value: String,
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
    fn get_enum_id(&self) -> sys::ValueType {
        sys::ValueType_NIX_TYPE_STRING
    }

    fn new(inner: NonNull<sys::nix_value>) -> Self {
        //     wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
        //         sys::nix_get_int(ctx.as_ptr(), inner.as_ptr())
        //     })
        //     .unwrap_or_else(|err| {
        //         panic_issue_call_failed!(
        //             "`sys::nix_get_int` failed for valid `NixString` ({})",
        //             err
        //         )
        //     })
        // };

        Self { inner, value }
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
