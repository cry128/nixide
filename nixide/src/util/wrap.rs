use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_uint, c_void};
use std::path::PathBuf;

use crate::errors::{ErrorContext, NixideError};
use crate::stdext::CCharPtrExt as _;
use crate::util::wrappers::AsInnerPtr;
use crate::NixideResult;

struct UserData<T> {
    // inner: T,
    inner: MaybeUninit<T>,
}

impl<T> AsInnerPtr<T> for UserData<T> {
    unsafe fn as_ptr(&self) -> *mut T {
        self.inner.as_mut_ptr()
    }
}

macro_rules! nonnull {
    ($ptr:expr $(,)? ) => {{
        match ::std::ptr::NonNull::new($ptr) {
            ::std::option::Option::Some(p) => ::std::result::Result::Ok(p),
            ::std::option::Option::None => {
                ::std::result::Result::Err(crate::errors::new_nixide_error!(NullPtr))
            }
        }
    }};
}
pub(crate) use nonnull;

macro_rules! nix_fn {
    ($callback:expr $(,)? ) => {{
        // XXX: TODO: what happens if i DO actually use `null_mut` instead of ErrorContext::new? does rust just panic?
        let mut ctx = crate::errors::ErrorContext::new();
        let result = $callback(
            &ctx,
        );
        ctx.pop().and_then(|_| ::std::result::Result::Ok(result))
    }};
}
pub(crate) use nix_fn;

macro_rules! nix_ptr_fn {
    ($callback:expr $(,)? ) => {{
        crate::util::wrap::nix_fn!($callback).and_then(|ptr| crate::util::wrap::nonnull!(ptr))
    }};
}
pub(crate) use nix_ptr_fn;

macro_rules! nix_callback {
    ( | $($arg_name:ident : $arg_type:ty),* ; userdata $( : *mut c_void )? $(,)? | -> $ret:ty $body:block, $callback:expr $(,)? ) => {{
        // create a function item that wraps the closure body (so it has a concrete type)
        #[allow(unused_variables)]
        fn __captured_fn( $( $arg_name: $arg_type ),*, userdata: *mut ::std::os::raw::c_void) -> $ret $body

        unsafe extern "C" fn __wrapper_callback(
            $(
              $arg_name: $arg_type,
            )*
            userdata: *mut ::std::os::raw::c_void,
        ) {
            let result = unsafe { &mut *(userdata as *mut $ret) };

            *result = __captured_fn(
                $(
                    $arg_name,
                )*
                userdata,
            );
        }

        // XXX: TODO: what happens if i DO actually use `null_mut` instead of ErrorContext::new? does rust just panic?
        let mut ctx = crate::errors::ErrorContext::new();
        let mut result: ::std::mem::MaybeUninit<$ret> = ::std::mem::MaybeUninit::uninit();

        $callback(
            __wrapper_callback,
            result.as_mut_ptr() as *mut ::std::os::raw::c_void,
            &ctx,
        );
        ctx.pop().and_then(|_| unsafe { result.assume_init() })

    }};

    ( | userdata $( : *mut c_void )? ; $($arg_name:ident : $arg_type:ty),* | -> $ret:ty $body:block, $callback:expr $(,)? ) => {{
        // create a function item that wraps the closure body (so it has a concrete type)
        #[allow(unused_variables)]
        fn __captured_fn(userdata: *mut ::std::os::raw::c_void, $($arg_name: $arg_type),*) -> $ret $body

        unsafe extern "C" fn __wrapper_callback(
            userdata: *mut ::std::os::raw::c_void,
            $(
              $arg_name: $arg_type,
            )*
        ) {
            let result = unsafe { &mut *(userdata as *mut $ret) };

            *result = __captured_fn(
                userdata,
                $(
                    $arg_name,
                )*
            );
        }

        // XXX: TODO: what happens if i DO actually use `null_mut` instead of ErrorContext::new? does rust just panic?
        let mut ctx = crate::errors::ErrorContext::new();
        let mut result: ::std::mem::MaybeUninit<$ret> = ::std::mem::MaybeUninit::uninit();

        $callback(
            __wrapper_callback,
            result.as_mut_ptr() as *mut ::std::os::raw::c_void,
            &ctx,
        );
        ctx.pop().and_then(|_| unsafe { result.assume_init() })
    }};
}
pub(crate) use nix_callback;

pub fn nix_string_callback<F>(callback: F) -> Result<String, NixideError>
where
    F: FnOnce(
        unsafe extern "C" fn(*const c_char, c_uint, *mut c_void),
        *mut c_void,
        &ErrorContext,
    ) -> i32,
{
    crate::util::wrap::nix_callback!(
        |start: *const c_char, n: c_uint ; userdata| -> NixideResult<String> {
            start.to_utf8_string_n(n as usize)
        },
        callback
    )
}

pub fn nix_pathbuf_callback<F>(callback: F) -> Result<PathBuf, NixideError>
where
    F: FnOnce(
        unsafe extern "C" fn(*const c_char, c_uint, *mut c_void),
        *mut c_void,
        &ErrorContext,
    ) -> i32,
{
    nix_string_callback(callback).map(::std::path::PathBuf::from)
}
