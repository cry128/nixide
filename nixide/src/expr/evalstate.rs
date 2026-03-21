use std::ffi::CString;
use std::ptr::NonNull;
use std::sync::Arc;

use crate::errors::new_nixide_error;

use super::Value;
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::{NixideError, Store};

/// Nix evaluation state for evaluating expressions.
///
/// This provides the main interface for evaluating Nix expressions
/// and creating values.
pub struct EvalState {
    inner: NonNull<sys::EvalState>,

    // XXX: TODO: is an `Arc<Store>` necessary or just a `Store`
    #[allow(dead_code)]
    store: Arc<Store>,
}

impl AsInnerPtr<sys::EvalState> for EvalState {
    unsafe fn as_ptr(&self) -> *mut sys::EvalState {
        self.inner.as_ptr()
    }
}

impl EvalState {
    /// Construct a new EvalState directly from its attributes
    pub(super) fn new(inner: NonNull<sys::EvalState>, store: Arc<Store>) -> Self {
        Self { inner, store }
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
    pub fn eval_from_string(&self, expr: &str, path: &str) -> Result<Value, NixideError> {
        let expr_c = CString::new(expr).or(Err(new_nixide_error!(StringNulByte)))?;
        let path_c = CString::new(path).or(Err(new_nixide_error!(StringNulByte)))?;

        let ctx = ErrorContext::new();
        // Allocate value for result
        // XXX: TODO: refactor this code to use `nixide::Value`
        let value_ptr = unsafe { sys::nix_alloc_value(ctx.as_ptr(), self.as_ptr()) };
        let value = match ctx.peak() {
            Some(err) => Err(err),
            None => match NonNull::new(value_ptr) {
                Some(inner) => Ok(Value { inner }),
                None => Err(new_nixide_error!(NullPtr)),
            },
        }?;

        // Evaluate expression
        unsafe {
            sys::nix_expr_eval_from_string(
                ctx.as_ptr(),
                self.as_ptr(),
                expr_c.as_ptr(),
                path_c.as_ptr(),
                value.as_ptr(),
            );
        };
        match ctx.peak() {
            Some(err) => Err(err),
            None => Ok(value),
        }
    }

    /// Allocate a new value.
    ///
    /// # Errors
    ///
    /// Returns an error if value allocation fails.
    pub fn alloc_value(&self) -> Result<Value, NixideError> {
        let ctx = ErrorContext::new();
        let value_ptr = unsafe { sys::nix_alloc_value(ctx.as_ptr(), self.as_ptr()) };
        match ctx.peak() {
            Some(err) => Err(err),
            None => match NonNull::new(value_ptr) {
                Some(inner) => Ok(Value { inner }),
                None => Err(new_nixide_error!(NullPtr)),
            },
        }
    }
}

impl Drop for EvalState {
    fn drop(&mut self) {
        // SAFETY: We own the state and it's valid until drop
        unsafe {
            sys::nix_state_free(self.inner.as_ptr());
        }
    }
}

// SAFETY: EvalState can be shared between threads
unsafe impl Send for EvalState {}
unsafe impl Sync for EvalState {}
