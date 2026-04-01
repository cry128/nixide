use std::cell::RefCell;
use std::ffi::{CString, c_char};
use std::ptr::{self, NonNull};
use std::rc::Rc;

use super::EvalState;
#[cfg(feature = "flakes")]
use crate::FlakeSettings;
use crate::Store;
use crate::errors::{ErrorContext, NixideResult};
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::util::{panic_issue_call_failed, wrap};

/// Builder for Nix evaluation state.
///
/// This allows configuring the evaluation environment before creating
/// the evaluation state.
///
pub struct EvalStateBuilder {
    inner: Rc<RefCell<NonNull<sys::NixEvalStateBuilder>>>,
    store: Rc<RefCell<Store>>,
}

impl AsInnerPtr<sys::NixEvalStateBuilder> for EvalStateBuilder {
    #[inline]
    unsafe fn as_ptr(&self) -> *mut sys::NixEvalStateBuilder {
        self.inner.borrow().as_ptr()
    }

    #[inline]
    unsafe fn as_ref(&self) -> &sys::NixEvalStateBuilder {
        unsafe { self.inner.borrow().as_ref() }
    }

    #[inline]
    unsafe fn as_mut(&mut self) -> &mut sys::NixEvalStateBuilder {
        unsafe { self.inner.borrow_mut().as_mut() }
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
    ///
    pub fn new(store: Rc<RefCell<Store>>) -> NixideResult<Self> {
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_eval_state_builder_new(ctx.as_ptr(), store.borrow().as_ptr())
        })?;

        Ok(EvalStateBuilder {
            inner: Rc::new(RefCell::new(inner)),
            store,
        })
    }

    /// Build the evaluation state.
    ///
    /// # Errors
    ///
    /// Returns an error if the evaluation state cannot be built.
    ///
    pub fn build(self) -> NixideResult<EvalState> {
        // Build the state
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_eval_state_build(ctx.as_ptr(), self.as_ptr())
        })?;

        Ok(EvalState::from(inner, self.store.clone()))
    }

    // XXX: TODO: use `flakes()` instead
    #[deprecated]
    #[cfg(feature = "flakes")]
    pub fn set_flake_settings(self, settings: &FlakeSettings) -> NixideResult<Self> {
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_flake_settings_add_to_eval_state_builder(
                ctx.as_ptr(),
                settings.as_ptr(),
                self.as_ptr(),
            );
        })?;

        Ok(self)
    }

    #[cfg(feature = "flakes")]
    pub fn flakes(self) -> NixideResult<Self> {
        #[allow(deprecated)]
        self.set_flake_settings(&FlakeSettings::new()?)
    }

    pub fn load_ambient_settings(self) -> NixideResult<Self> {
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_eval_state_builder_load(ctx.as_ptr(), self.as_ptr());
        })?;

        Ok(self)
    }

    pub fn set_lookup_path<P: AsRef<str>>(self, paths: Vec<P>) -> NixideResult<Self> {
        let paths_len = paths.len();
        let paths_capacity = paths.capacity();

        // XXX: TODO: use the `AsCArray` trait instead
        let mut ptrs: Vec<*const c_char> = paths
            .into_iter()
            .map(|p| {
                CString::new(p.as_ref())
                    .unwrap_or_else(|err| {
                        panic_issue_call_failed!(
                            "given string {} contains a NUL byte ({})",
                            p.as_ref(),
                            err
                        )
                    })
                    .into_raw() as *const c_char
            })
            .collect();

        ptrs.push(ptr::null());

        // Leak the Vec and return as mutable pointer
        let ptr = ptrs.as_mut_ptr();
        std::mem::forget(ptrs);

        let result = wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_eval_state_builder_set_lookup_path(ctx.as_ptr(), self.as_ptr(), ptr);
        })
        .map(|()| self);

        // ensure all allocated memory is dropped
        // XXX: TODO!!
        // unsafe {
        //     Vec::from_raw_parts(ptr, paths_len, paths_capacity)
        //         .into_iter()
        //         .map(|p| {
        //             _ = CString::from_raw(p as *mut c_char);
        //         })
        // };

        result
    }
}

impl Drop for EvalStateBuilder {
    fn drop(&mut self) {
        unsafe {
            sys::nix_eval_state_builder_free(self.as_ptr());
        }
    }
}
