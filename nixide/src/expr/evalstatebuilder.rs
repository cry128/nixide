use std::ptr::NonNull;
use std::sync::Arc;

use super::EvalState;
use crate::errors::{ErrorContext, NixideResult};
use crate::sys;
use crate::util::wrap;
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

impl AsInnerPtr<sys::nix_eval_state_builder> for EvalStateBuilder {
    #[inline]
    unsafe fn as_ptr(&self) -> *mut sys::nix_eval_state_builder {
        self.inner.as_ptr()
    }

    #[inline]
    unsafe fn as_ref(&self) -> &sys::nix_eval_state_builder {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    unsafe fn as_mut(&mut self) -> &mut sys::nix_eval_state_builder {
        unsafe { self.inner.as_mut() }
    }
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
    pub fn new(store: &Arc<Store>) -> NixideResult<Self> {
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_eval_state_builder_new(ctx.as_ptr(), store.as_ptr())
        })?;

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
    pub fn build(self) -> NixideResult<EvalState> {
        // Load configuration first
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_eval_state_builder_load(ctx.as_ptr(), self.as_ptr())
        })?;

        // Build the state
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_eval_state_build(ctx.as_ptr(), self.as_ptr())
        })?;

        Ok(EvalState::new(inner, self.store.clone()))
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
