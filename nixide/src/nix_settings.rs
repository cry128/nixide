use std::ffi::c_void;

use crate::NixideResult;
use crate::errors::ErrorContext;
use crate::stdext::AsCPtr as _;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr as _;

/// # Note
/// This function is intentionally marked unsafe to discourage its use.
/// Please prefer [nixide::FlakeSettings] and [nixide::FetchersSettings].
///
pub unsafe fn get_global_setting<S: AsRef<str>>(key: S) -> NixideResult<String> {
    let key = key.as_c_ptr()?;

    wrap::nix_string_callback!(
        |callback, userdata: *mut __UserData, ctx: &ErrorContext| unsafe {
            sys::nix_setting_get(ctx.as_ptr(), key, Some(callback), userdata as *mut c_void);
        }
    )
}

/// # Note
/// This function is intentionally marked unsafe to discourage its use.
/// Please prefer [nixide::FlakeSettings] and [nixide::FetchersSettings].
///
pub unsafe fn set_global_setting<S: AsRef<str>, T: AsRef<str>>(
    key: S,
    value: T,
) -> NixideResult<()> {
    let key = key.as_c_ptr()?;
    let value = value.as_c_ptr()?;

    wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
        sys::nix_setting_set(ctx.as_ptr(), key, value);
    })
}
