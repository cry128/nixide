#[repr(C)]
#[derive(Debug)]
pub(crate) struct UserData<S, T> {
    pub inner: S,
    pub retval: T,
}

impl<S, T> AsMut<UserData<S, T>> for UserData<S, T> {
    fn as_mut(&mut self) -> &mut UserData<S, T> {
        self
    }
}

impl<S, T> UserData<S, T> {
    /// # Warning
    ///
    /// Ensure `self.retval` has been initialised before unwrapping!
    ///
    pub unsafe fn unwrap(self) -> (S, T) {
        (self.inner, self.retval)
    }

    pub unsafe fn as_mut_ptr(&mut self) -> *mut Self {
        self as *mut Self
    }

    pub unsafe fn inner_ptr(&mut self) -> *mut S {
        unsafe {
            let ptr = self.as_mut_ptr();
            &raw mut (*ptr).inner
        }
    }

    // pub unsafe fn retval_ptr(&mut self) -> *mut c_void {
    //     &mut self.retval as *mut T as *mut c_void
    // }
}

macro_rules! nonnull {
    ($ptr:expr $(,)? ) => {{
        match ::std::ptr::NonNull::new($ptr) {
            ::std::option::Option::Some(p) => ::std::result::Result::Ok(p),
            ::std::option::Option::None => {
                ::std::result::Result::Err($crate::errors::new_nixide_error!(NullPtr))
            }
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
    ($callback:expr $(,)? ) => {{
        $crate::util::wrap::nix_fn!($callback).and_then(|ptr| $crate::util::wrap::nonnull!(ptr))
    }};
}
pub(crate) use nix_ptr_fn;

macro_rules! __nix_callback {
    ($userdata_type:ty, $ret:ty, $callback:expr) => {{
        let mut __ctx = $crate::errors::ErrorContext::new();
        let mut __state: ::std::mem::MaybeUninit<__UserData> = ::std::mem::MaybeUninit::uninit();

        $callback(__wrapper_callback, __state.as_mut_ptr(), &__ctx);

        // add type annotations for compiler
        let __return: $ret = __ctx
            .pop()
            .and_then(|_| unsafe { __state.assume_init().retval });
        __return
    }};
}
pub(crate) use __nix_callback;

/// `libnix` functions consistently either expect the `userdata`/`user_data` (inconsistently named in the API...)
/// field to be the first or last parameter (differs between function). The `nix_callback!` macro allows the
/// position to be specified by either the following syntax:
///
/// ```rs
/// nix_callback(userdata; ...); // first parameter
/// nix_callback(...; userdata); // last parameter
/// ```
///
macro_rules! nix_callback {
    ( | $userdata:ident : $userdata_type:ty; $($arg_name:ident : $arg_type:ty),* $(,)? | -> $ret:ty $body:block, $function:expr $(,)? ) => {{
        type __UserData = $crate::util::wrap::UserData<$userdata_type, $ret>;
        // create a function item that wraps the closure body (so it has a concrete type)
        fn __captured_fn($userdata: &mut __UserData, $($arg_name: $arg_type),*) -> $ret $body

        unsafe extern "C" fn __wrapper_callback(
            $userdata: *mut ::std::ffi::c_void,
            $(
              $arg_name: $arg_type,
            )*
        ) {
            let ud = unsafe { &mut *($userdata as *mut __UserData) };
            let stored_retval = &raw mut ud.retval;

            let retval = __captured_fn(
                ud,
                $(
                    $arg_name,
                )*
            );

            unsafe {
                stored_retval.write(retval)
            };
        }

        let mut __ctx: $crate::errors::ErrorContext = $crate::errors::ErrorContext::new();
        let mut __state: ::std::mem::MaybeUninit<__UserData> = ::std::mem::MaybeUninit::uninit();

        // fn __captured_function(
        //     callback: unsafe extern "C" fn(
        //         $userdata: *mut ::std::ffi::c_void,
        //         $(
        //           $arg_name: $arg_type,
        //         )*
        //     ),
        //     state: *mut __UserData,
        //     ctx: &$crate::errors::ErrorContext,
        // ) {
        //     $function(callback, state, ctx);
        // }

        $function(__wrapper_callback, __state.as_mut_ptr(), &__ctx);

        // add type annotations for compiler
        __ctx.pop().and_then(|_| unsafe { __state.assume_init().retval })
    }};

    ( | $($arg_name:ident : $arg_type:ty),* ; $userdata:ident : $userdata_type:ty $(,)? | -> $ret:ty $body:block, $callback:expr $(,)? ) => {{
        type __UserData = $crate::util::wrap::UserData<$userdata_type, $ret>;
        // create a function item that wraps the closure body (so it has a concrete type)
        unsafe extern "C" fn __captured_fn( $( $arg_name: $arg_type ),*, $userdata: &mut __UserData) -> $ret $body

        unsafe extern "C" fn __wrapper_callback(
            $(
              $arg_name: $arg_type,
            )*
            $userdata: *mut ::std::ffi::c_void,
        ) {
            unsafe {
                let ud = &mut *($userdata as *mut __UserData);
                let stored_retval = &raw mut ud.retval;

                let retval = __captured_fn(
                    $(
                        $arg_name,
                    )*
                    ud,
                );

                stored_retval.write(retval)
            }
        }

        // $crate::util::wrap::__nix_callback!($userdata_type, $ret, $callback)
        let mut __ctx = $crate::errors::ErrorContext::new();
        let mut __state: ::std::mem::MaybeUninit<
            __UserData
        > = ::std::mem::MaybeUninit::uninit();

        $callback(__wrapper_callback, __state.as_mut_ptr(), &__ctx);

        // add type annotations for compiler
        let __return: $ret = __ctx.pop().and_then(|_| unsafe { __state.assume_init().retval });
        __return
    }};
}
pub(crate) use nix_callback;

// XXX: TODO: convert these to declarative macros
macro_rules! nix_string_callback {
    ($callback:expr $(,)?) => {{
        $crate::util::wrap::nix_callback!(
            |start: *const ::std::ffi::c_char, n: ::std::ffi::c_uint ; userdata: ()| -> $crate::NixideResult<String> {
                start.to_utf8_string_n(n as usize)
            },
            $callback
        )
    }};
}
pub(crate) use nix_string_callback;

macro_rules! nix_pathbuf_callback {
    ($callback:expr $(,)?) => {{
        $crate::util::wrap::nix_string_callback!($callback).map(::std::path::PathBuf::from)
    }};
}
pub(crate) use nix_pathbuf_callback;
