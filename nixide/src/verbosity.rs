use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrappers::AsInnerPtr as _;
use crate::util::{panic_issue, panic_issue_call_failed, wrap};

/// Verbosity level
///
/// # NOTE
///
/// This should be kept in sync with the C++ implementation (nix::Verbosity)
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

impl From<sys::nix_verbosity> for NixVerbosity {
    fn from(level: sys::nix_verbosity) -> NixVerbosity {
        match level {
            sys::nix_verbosity_NIX_LVL_ERROR => NixVerbosity::Error,
            sys::nix_verbosity_NIX_LVL_WARN => NixVerbosity::Warn,
            sys::nix_verbosity_NIX_LVL_NOTICE => NixVerbosity::Notice,
            sys::nix_verbosity_NIX_LVL_INFO => NixVerbosity::Info,
            sys::nix_verbosity_NIX_LVL_TALKATIVE => NixVerbosity::Talkative,
            sys::nix_verbosity_NIX_LVL_CHATTY => NixVerbosity::Chatty,
            sys::nix_verbosity_NIX_LVL_DEBUG => NixVerbosity::Debug,
            sys::nix_verbosity_NIX_LVL_VOMIT => NixVerbosity::Vomit,
            value => panic_issue!(
                "nixide encountered unknown `nix_verbosity` value ({})",
                value
            ),
        }
    }
}

impl Into<sys::nix_verbosity> for NixVerbosity {
    fn into(self) -> sys::nix_verbosity {
        self as sys::nix_verbosity
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
