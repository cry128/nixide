use std::ffi::c_char;
use std::ptr::NonNull;
use std::sync::Arc;

use crate::errors::ErrorContext;
use crate::expr::values::NixString;
use crate::stdext::CCharPtrExt;
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::util::LazyArray;
use crate::util::{panic_issue_call_failed, wrap};
use crate::{EvalState, NixideResult, StorePath};

pub struct RealisedString {
    inner: NonNull<sys::nix_realised_string>,
    // pub path: LazyCell<StorePath, Box<fn() -> StorePath>>,
    pub path: StorePath,
    pub children: LazyArray<StorePath, fn(&LazyArray<StorePath, fn(usize) -> StorePath>, usize) -> StorePath>>,
}

impl AsInnerPtr<sys::nix_realised_string> for RealisedString {
    #[inline]
    unsafe fn as_ptr(&self) -> *mut sys::nix_realised_string {
        self.inner.as_ptr()
    }

    #[inline]
    unsafe fn as_ref(&self) -> &sys::nix_realised_string {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    unsafe fn as_mut(&mut self) -> &mut sys::nix_realised_string {
        unsafe { self.inner.as_mut() }
    }
}

impl Drop for RealisedString {
    fn drop(&mut self) {
        unsafe {
            sys::nix_realised_string_free(self.as_ptr());
        }
    }
}

impl RealisedString {
    /// Realise a string context.
    ///
    /// This will
    ///  - realise the store paths referenced by the string's context, and
    ///  - perform the replacement of placeholders.
    ///  - create temporary garbage collection roots for the store paths, for
    ///    the lifetime of the current process.
    ///  - log to stderr
    ///
    /// # Arguments
    ///
    /// * value - Nix value, which must be a string
    /// * state - Nix evaluator state
    /// * isIFD - If true, disallow derivation outputs if setting `allow-import-from-derivation` is false.
    ///           You should set this to true when this call is part of a primop.
    ///           You should set this to false when building for your application's purpose.
    ///
    /// # Returns
    ///
    /// NULL if failed, or a new nix_realised_string, which must be freed with nix_realised_string_free
    pub fn new(value: &NixString, state: &Arc<EvalState>) -> NixideResult<Self> {
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_string_realise(
                ctx.as_ptr(),
                state.as_ptr(),
                value.as_ptr(),
                false, // don't copy more
            )
        })?;

        fn delegate(
            inner: &LazyArray<StorePath, Box<dyn Fn(usize) -> StorePath>>,
            index: usize,
        ) -> StorePath {
            // XXX: TODO
            // inner[index]
            StorePath::fake_path(unsafe { state.store_ref() }).unwrap()
        }

        let size = unsafe { sys::nix_realised_string_get_store_path_count(inner.as_ptr()) };

        Ok(Self {
            inner,
            path: Self::parse_path(inner.as_ptr(), state),
            // children: LazyArray::new(size, delegate as fn(usize) -> StorePath),
            children: LazyArray::<StorePath, Box<dyn Fn(usize) -> StorePath>>::new(
                size,
                Box::new(delegate),
            ),
        })
    }

    fn parse_path(
        realised_string: *mut sys::nix_realised_string,
        state: &Arc<EvalState>,
    ) -> StorePath {
        let buffer_ptr = unsafe { sys::nix_realised_string_get_buffer_start(realised_string) };
        let buffer_size = unsafe { sys::nix_realised_string_get_buffer_size(realised_string) };

        let path_str = (buffer_ptr as *const c_char)
            .to_utf8_string_n(buffer_size)
            .unwrap_or_else(|err| {
                panic_issue_call_failed!(
                    "`sys::nix_realised_string_get_buffer_(start|size)` invalid UTF-8 ({})",
                    err
                )
            });
        StorePath::parse(unsafe { state.store_ref() }, &path_str).unwrap_or_else(|err| {
            panic_issue_call_failed!(
                "`sys::nix_realised_string_get_buffer_(start|size)` invalid store path ({})",
                err
            )
        })
    }
}
