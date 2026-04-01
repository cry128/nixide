use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrappers::AsInnerPtr as _;
use crate::util::{panic_issue_call_failed, wrap};

/// Verbosity level
///
/// # NOTE
///
/// This should be kept in sync with the C++ implementation (nix::Verbosity)
///
#[derive(Debug, Clone, Copy)]
pub enum NixVerbosity {
    Error,
    Warn,
    Notice,
    Info,
    Talkative,
    Chatty,
    Debug,
    Vomit,
}

impl From<sys::NixVerbosity> for NixVerbosity {
    fn from(level: sys::NixVerbosity) -> NixVerbosity {
        match level {
            sys::NixVerbosity::Error => NixVerbosity::Error,
            sys::NixVerbosity::Warn => NixVerbosity::Warn,
            sys::NixVerbosity::Notice => NixVerbosity::Notice,
            sys::NixVerbosity::Info => NixVerbosity::Info,
            sys::NixVerbosity::Talkative => NixVerbosity::Talkative,
            sys::NixVerbosity::Chatty => NixVerbosity::Chatty,
            sys::NixVerbosity::Debug => NixVerbosity::Debug,
            sys::NixVerbosity::Vomit => NixVerbosity::Vomit,
        }
    }
}

impl Into<sys::NixVerbosity> for NixVerbosity {
    fn into(self) -> sys::NixVerbosity {
        self as sys::NixVerbosity
    }
}

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
        sys::nix_set_verbosity(ctx.as_ptr(), level.into());
    })
    .unwrap_or_else(|err| panic_issue_call_failed!("{}", err))
}
