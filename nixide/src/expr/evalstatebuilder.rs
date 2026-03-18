use std::ptr::NonNull;
use std::sync::Arc;

use super::EvalState;
use crate::sys;
use crate::{ErrorContext, NixErrorCode, Store};

/// Builder for Nix evaluation state.
///
/// This allows configuring the evaluation environment before creating
/// the evaluation state.
pub struct EvalStateBuilder {
    inner: NonNull<sys::nix_eval_state_builder>,
    store: Arc<Store>,
    context: Arc<ErrorContext>,
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
    pub fn new(store: &Arc<Store>) -> Result<Self, NixErrorCode> {
        // SAFETY: store context and store are valid
        let builder_ptr =
            unsafe { sys::nix_eval_state_builder_new(store._context.as_ptr(), store.as_ptr()) };

        let inner = NonNull::new(builder_ptr).ok_or(NixErrorCode::NullPtr {
            location: "nix_eval_state_builder_new",
        })?;

        Ok(EvalStateBuilder {
            inner,
            store: Arc::clone(store),
            context: Arc::clone(&store._context),
        })
    }

    /// Build the evaluation state.
    ///
    /// # Errors
    ///
    /// Returns an error if the evaluation state cannot be built.
    pub fn build(self) -> Result<EvalState, NixErrorCode> {
        // Load configuration first
        // SAFETY: context and builder are valid
        NixErrorCode::from(
            unsafe { sys::nix_eval_state_builder_load(self.context.as_ptr(), self.inner.as_ptr()) },
            "nix_eval_state_builder_load",
        )?;

        // Build the state
        // SAFETY: context and builder are valid
        let state_ptr =
            unsafe { sys::nix_eval_state_build(self.context.as_ptr(), self.inner.as_ptr()) };

        let inner = NonNull::new(state_ptr).ok_or(NixErrorCode::NullPtr {
            location: "nix_eval_state_build",
        })?;

        // The builder is consumed here - its Drop will clean up
        Ok(EvalState::new(
            inner,
            self.store.clone(),
            self.context.clone(),
        ))
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
