use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr as _;

pub use sys::NixVerbosity;

/// Sets the verbosity level.
///
/// **This function should never fail!**
/// A panic would indicate a bug in nixide itself.
///
/// # Nix C++ API Internals
///
/// ```cpp
/// nix_err nix_set_verbosity(nix_c_context * context, nix_verbosity level)
/// {
///     if (context)
///         context->last_err_code = NIX_OK;
///     if (level > NIX_LVL_VOMIT || level < NIX_LVL_ERROR)
///         return nix_set_err_msg(context, NIX_ERR_UNKNOWN, "Invalid verbosity level");
///     try {
///         nix::verbosity = static_cast<nix::Verbosity>(level);
///     } catch (...) {
///         return nix_context_error(context);
///     }
///     return NIX_OK;
/// }
/// ```
///
pub fn set_verbosity(level: NixVerbosity) {
    wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
        sys::nix_set_verbosity(ctx.as_ptr(), level);
    })
    .unwrap()
}
