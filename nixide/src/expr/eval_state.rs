use std::cell::RefCell;
use std::ptr::NonNull;
use std::rc::Rc;

use crate::stdext::AsCPtr as _;

use super::Value;
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::util::{panic_issue_call_failed, wrap};
use crate::{NixideResult, Store};

/// Nix evaluation state for evaluating expressions.
///
/// This provides the main interface for evaluating Nix expressions
/// and creating values.
#[derive(Clone)]
pub struct EvalState {
    inner: Rc<RefCell<NonNull<sys::EvalState>>>,

    store: Rc<RefCell<Store>>,
}

// impl Clone for EvalState {
//     fn clone(&self) -> Self {
//         let inner = self.inner.clone();
//
//         wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
//             sys::nix_gc_incref(ctx.as_ptr(), self.as_ptr() as *mut c_void);
//         })
//         .unwrap();
//
//         Self {
//             inner,
//             store: self.store.clone(),
//         }
//     }
// }

impl AsInnerPtr<sys::EvalState> for EvalState {
    #[inline]
    unsafe fn as_ptr(&self) -> *mut sys::EvalState {
        self.inner.borrow().as_ptr()
    }

    #[inline]
    unsafe fn as_ref(&self) -> &sys::EvalState {
        unsafe { self.inner.borrow().as_ref() }
    }

    #[inline]
    unsafe fn as_mut(&mut self) -> &mut sys::EvalState {
        unsafe { self.inner.borrow_mut().as_mut() }
    }
}

impl EvalState {
    /// Construct a new EvalState directly from its attributes
    ///
    pub(super) fn from(inner: NonNull<sys::EvalState>, store: Rc<RefCell<Store>>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(inner)),
            store,
        }
    }

    #[inline]
    pub fn inner_ref(&self) -> &Rc<RefCell<NonNull<sys::EvalState>>> {
        &self.inner
    }

    #[inline]
    pub fn store_ref(&self) -> &Rc<RefCell<Store>> {
        &self.store
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
    ///
    pub fn interpret(&self, expr: &str, path: &str) -> NixideResult<Value> {
        let expr = expr.as_c_ptr()?;
        let path = path.as_c_ptr()?;

        // Allocate value for result
        // XXX: TODO: create a method for this (``)
        let value = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_alloc_value(ctx.as_ptr(), self.as_ptr())
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        // Evaluate expression
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_expr_eval_from_string(ctx.as_ptr(), self.as_ptr(), expr, path, value.as_ptr());
            value
        })
        .map(|ptr| Value::from((ptr, self.inner_ref().clone())))
    }
}

impl Drop for EvalState {
    fn drop(&mut self) {
        unsafe {
            sys::nix_state_free(self.as_ptr());
        }
    }
}
