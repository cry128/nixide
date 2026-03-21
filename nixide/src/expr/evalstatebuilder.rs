use std::ptr::NonNull;
use std::sync::Arc;

use super::EvalState;
use crate::errors::{new_nixide_error, ErrorContext, NixideError};
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::Store;

/// Builder for Nix evaluation state.
///
/// This allows configuring the evaluation environment before creating
/// the evaluation state.
pub struct EvalStateBuilder {
    inner: NonNull<sys::nix_eval_state_builder>,
    store: Arc<Store>,
}

impl EvalStateBuilder {
    /// Create a new evaluation state builder.
    ///
    /// # Arguments
    ///
    /// * `store` - The Nix store to use for evaluation
    ///
    /// # Errors
    ///
    /// Returns an error if the builder cannot be created.
    pub fn new(store: &Arc<Store>) -> Result<Self, NixideError> {
        // SAFETY: store context and store are valid
        let builder_ptr =
            unsafe { sys::nix_eval_state_builder_new(store._context.as_ptr(), store.as_ptr()) };

        let inner = NonNull::new(builder_ptr).ok_or(new_nixide_error!(NullPtr))?;

        Ok(EvalStateBuilder {
            inner,
            store: Arc::clone(store),
        })
    }

    /// Build the evaluation state.
    ///
    /// # Errors
    ///
    /// Returns an error if the evaluation state cannot be built.
    pub fn build(self) -> Result<EvalState, NixideError> {
        let ctx = ErrorContext::new();
        // Load configuration first
        unsafe { sys::nix_eval_state_builder_load(ctx.as_ptr(), self.as_ptr()) };
        if let Some(err) = ctx.peak() {
            return Err(err);
        }

        // Build the state
        let state_ptr = unsafe { sys::nix_eval_state_build(ctx.as_ptr(), self.as_ptr()) };
        if let Some(err) = ctx.peak() {
            return Err(err);
        }

        let inner = NonNull::new(state_ptr).ok_or(new_nixide_error!(NullPtr))?;

        // The builder is consumed here - its Drop will clean up
        Ok(EvalState::new(inner, self.store.clone()))
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut sys::nix_eval_state_builder {
        self.inner.as_ptr()
    }
}

impl Drop for EvalStateBuilder {
    fn drop(&mut self) {
        // SAFETY: We own the builder and it's valid until drop
        unsafe {
            sys::nix_eval_state_builder_free(self.inner.as_ptr());
        }
    }
}
