// #![warn(missing_docs)]

#![cfg_attr(nightly, feature(fn_traits))]
#![cfg_attr(nightly, feature(unboxed_closures))]

pub extern crate libc;
pub extern crate nixide_sys as sys;

pub(crate) mod errors;
mod init;
mod nix_settings;
mod stdext;
pub(crate) mod util;
mod verbosity;
mod version;

#[cfg(feature = "exprs")]
mod expr;
#[cfg(feature = "flakes")]
mod flake;
#[cfg(feature = "store")]
mod store;

pub use errors::{NixError, NixideError, NixideResult};
pub use nix_settings::{get_global_setting, set_global_setting};
pub use verbosity::{NixVerbosity, set_verbosity};
pub use version::NixVersion;

#[cfg(feature = "exprs")]
pub use expr::{EvalState, EvalStateBuilder, Value};
#[cfg(feature = "flakes")]
pub use flake::{FlakeSettings, LockedFlake};
#[cfg(feature = "store")]
pub use store::{Store, StorePath};
