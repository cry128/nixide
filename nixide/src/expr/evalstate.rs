use std::ffi::CString;
use std::ptr::NonNull;
use std::sync::Arc;

use crate::errors::new_nixide_error;

use super::Value;
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr;
use crate::{NixideResult, Store};

/// Nix evaluation state for evaluating expressions.
///
/// This provides the main interface for evaluating Nix expressions
/// and creating values.
pub struct EvalState {
    inner: NonNull<sys::EvalState>,

    // XXX: TODO: is an `Arc<Store>` necessary or just a `Store`
    store: Arc<Store>,
}

impl AsInnerPtr<sys::EvalState> for EvalState {
    #[inline]
    unsafe fn as_ptr(&self) -> *mut sys::EvalState {
        self.inner.as_ptr()
    }

    #[inline]
    unsafe fn as_ref(&self) -> &sys::EvalState {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    unsafe fn as_mut(&mut self) -> &mut sys::EvalState {
        unsafe { self.inner.as_mut() }
    }
}

impl EvalState {
    /// Construct a new EvalState directly from its attributes
    pub(super) fn new(inner: NonNull<sys::EvalState>, store: Arc<Store>) -> Self {
        Self { inner, store }
    }

    #[inline]
    pub(crate) unsafe fn store_ref(&self) -> &Store {
        self.store.as_ref()
    }

    /// Evaluate a Nix expression from a string.
    ///
    /// # Arguments
    ///
    /// * `expr` - The Nix expression to evaluate
    /// * `path` - The path to use for error reporting (e.g., "<eval>")
    ///
    /// # Errors
    ///
    /// Returns an error if evaluation fails.
    pub fn eval_from_string(&self, expr: &str, path: &str) -> NixideResult<Value> {
        let expr_c = CString::new(expr).or(Err(new_nixide_error!(StringNulByte)))?;
        let path_c = CString::new(path).or(Err(new_nixide_error!(StringNulByte)))?;

        // Allocate value for result
        let value = self.new_value()?;

        // Evaluate expression
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_expr_eval_from_string(
                ctx.as_ptr(),
                self.as_ptr(),
                expr_c.as_ptr(),
                path_c.as_ptr(),
                value.as_ptr(),
            );
            value
        })
    }

    /// Allocate a new value.
    ///
    /// # Errors
    ///
    /// Returns an error if value allocation fails.
    pub(self) fn new_value(&self) -> NixideResult<Value> {
        // XXX: TODO: should this function be `Value::new` instead?
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_alloc_value(ctx.as_ptr(), self.as_ptr())
        })?;

        Ok(Value::new(inner, self))
    }
}

impl Drop for EvalState {
    fn drop(&mut self) {
        unsafe {
            sys::nix_state_free(self.as_ptr());
        }
    }
}

// SAFETY: EvalState can be shared between threads
unsafe impl Send for EvalState {}
unsafe impl Sync for EvalState {}
