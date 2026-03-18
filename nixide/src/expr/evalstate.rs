use std::ffi::CString;
use std::ptr::NonNull;
use std::sync::Arc;

use super::Value;
use crate::sys;
use crate::{Context, NixError, Store};

/// Nix evaluation state for evaluating expressions.
///
/// This provides the main interface for evaluating Nix expressions
/// and creating values.
pub struct EvalState {
    inner: NonNull<sys::EvalState>,
    #[allow(dead_code)]
    store: Arc<Store>,
    pub(super) context: Arc<Context>,
}

impl EvalState {
    /// Construct a new EvalState directly from its attributes
    pub(super) fn new(
        inner: NonNull<sys::EvalState>,
        store: Arc<Store>,
        context: Arc<Context>,
    ) -> Self {
        Self {
            inner,
            store,
            context,
        }
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
    pub fn eval_from_string(&self, expr: &str, path: &str) -> Result<Value<'_>, NixError> {
        let expr_c =
            NixError::from_nulerror(CString::new(expr), "nixide::EvalState::eval_from_string")?;
        let path_c =
            NixError::from_nulerror(CString::new(path), "nixide::EvalState::eval_from_string")?;

        // Allocate value for result
        // SAFETY: context and state are valid
        let value_ptr = unsafe { sys::nix_alloc_value(self.context.as_ptr(), self.inner.as_ptr()) };
        if value_ptr.is_null() {
            return Err(NixError::NullPtr {
                location: "nix_alloc_value",
            });
        }

        // Evaluate expression
        // SAFETY: all pointers are valid
        NixError::from(
            unsafe {
                sys::nix_expr_eval_from_string(
                    self.context.as_ptr(),
                    self.inner.as_ptr(),
                    expr_c.as_ptr(),
                    path_c.as_ptr(),
                    value_ptr,
                )
            },
            "nix_expr_eval_from_string",
        )?;

        let inner = NonNull::new(value_ptr).ok_or(NixError::NullPtr {
            location: "nix_expr_eval_from_string",
        })?;

        Ok(Value { inner, state: self })
    }

    /// Allocate a new value.
    ///
    /// # Errors
    ///
    /// Returns an error if value allocation fails.
    pub fn alloc_value(&self) -> Result<Value<'_>, NixError> {
        // SAFETY: context and state are valid
        let value_ptr = unsafe { sys::nix_alloc_value(self.context.as_ptr(), self.inner.as_ptr()) };
        let inner = NonNull::new(value_ptr).ok_or(NixError::NullPtr {
            location: "nix_alloc_value",
        })?;

        Ok(Value { inner, state: self })
    }

    /// Get the raw state pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure the pointer is used safely.
    pub(super) unsafe fn as_ptr(&self) -> *mut sys::EvalState {
        self.inner.as_ptr()
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
