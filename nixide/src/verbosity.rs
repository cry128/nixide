use crate::sys;

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
            _ => panic!("nixide encountered unknown `nix_verbosity` value, please submit this as an issue at https://github.com/cry128/nixide"),
        }
    }
}
