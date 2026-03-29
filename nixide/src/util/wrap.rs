#[repr(C)]
#[derive(Debug)]
pub(crate) struct UserData<S, T> {
    pub inner: S,
    pub retval: T,

    // XXX: TODO: write impl functions to set and get these values,
    // XXX: TODO: and another one to unwrap a `MaybeUninit<UserData<S, T>>`
    #[cfg(debug_assertions)]
    pub init_inner: bool,

    #[cfg(debug_assertions)]
    pub init_retval: bool,
}

impl<S, T> AsMut<UserData<S, T>> for UserData<S, T> {
    fn as_mut(&mut self) -> &mut UserData<S, T> {
        self
    }
}

impl<S, T> UserData<S, T> {
    pub unsafe fn as_mut_ptr(&mut self) -> *mut Self {
        self as *mut Self
    }

    pub unsafe fn inner_ptr(&mut self) -> *mut S {
        unsafe {
            let ptr = self.as_mut_ptr();
            &raw mut (*ptr).inner
        }
    }
}

macro_rules! nonnull {
    ($ptr:expr $(,)? ) => {{
        match ::std::ptr::NonNull::new($ptr) {
            ::std::option::Option::Some(p) => ::std::result::Result::Ok(p),
            ::std::option::Option::None => {
                ::std::result::Result::Err($crate::errors::new_nixide_error!(NullPtr))
            },
        }
    }};
}

pub(crate) use nonnull;

macro_rules! nix_fn {
    ($callback:expr $(,)? ) => {{
        let mut __ctx = $crate::errors::ErrorContext::new();
        let __result = $callback(&__ctx);
        __ctx
            .pop()
            .and_then(|_| ::std::result::Result::Ok(__result))
    }};
}
pub(crate) use nix_fn;

macro_rules! nix_ptr_fn {
    ($callback:expr $(,)? ) => {{ $crate::util::wrap::nix_fn!($callback).and_then(|ptr| $crate::util::wrap::nonnull!(ptr)) }};
}
pub(crate) use nix_ptr_fn;

/// `libnix` functions consistently either expect the `userdata`/`user_data` (inconsistently named in the API...)
/// field to be the first or last parameter (differs between function). The `nix_callback!` macro allows the
/// position to be specified by either the following syntax:
///
/// ```rs
/// nix_callback!(; userdata; ...); // first parameter
/// nix_callback!(...; userdata; ); // last parameter
/// ```
///
macro_rules! nix_callback {
    ( | $( $($pre:ident : $pre_ty:ty),+ $(,)? )? ; $userdata:ident : $userdata_type:ty ; $( $($post:ident : $post_ty:ty),+ $(,)? )? | -> $ret:ty $body:block, $function:expr $(,)? ) => {{
        type __UserData = $crate::util::wrap::UserData<$userdata_type, $ret>;
        // create a function item that wraps the closure body (so it has a concrete type)
        // WARNING: this function must have no return type, use the `UserData.inner`
        // WARNING: field instead as an `out` pointer.
        #[allow(unused_variables)]
        unsafe extern "C" fn __captured_fn(
            $($( $pre: $pre_ty, )*)?
            $userdata: *mut __UserData,
            $($( $post: $post_ty, )*)?
        ) { $body }

        unsafe extern "C" fn __wrapper_callback(
            $($( $pre: $pre_ty, )*)?
            $userdata: *mut ::std::ffi::c_void,
            $($( $post: $post_ty, )*)?
        ) {
            unsafe {
                __captured_fn(
                    $($( $pre, )*)?
                    // userdata_,
                    $userdata as *mut __UserData,
                    $($( $post, )*)?
                );
            }
        }

        let mut __ctx: $crate::errors::ErrorContext = $crate::errors::ErrorContext::new();
        let mut __state: ::std::mem::MaybeUninit<__UserData> = ::std::mem::MaybeUninit::zeroed();

        $function(__wrapper_callback, __state.as_mut_ptr(), &__ctx);

        // type annotations for compiler
        let __return: $crate::NixideResult<$ret> = __ctx.pop().and_then(|_| ::std::result::Result::Ok(unsafe { __state.assume_init().retval }));
        __return
    }};
}
pub(crate) use nix_callback;

macro_rules! nix_string_callback {
    ($function:expr $(,)?) => {{
        let __result = $crate::util::wrap::nix_callback!(
            |start: *const ::std::ffi::c_char, n: ::std::ffi::c_uint; userdata: ();| -> $crate::NixideResult<String> {
                unsafe {
                    let retval = &raw mut (*userdata).retval;
                    retval.write($crate::stdext::CCharPtrExt::to_utf8_string_n(start, n as usize))
                }
            },
            $function
        );

        match __result {
            Ok(res) => res,
            Err(res) => Err(res),
        }
    }};
}
pub(crate) use nix_string_callback;

macro_rules! nix_pathbuf_callback {
    ($function:expr $(,)?) => {{ $crate::util::wrap::nix_string_callback!($function).map(::std::path::PathBuf::from) }};
}
pub(crate) use nix_pathbuf_callback;
