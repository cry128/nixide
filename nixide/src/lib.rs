// #![warn(missing_docs)]

mod context;
mod error;
mod expr;
mod flake;
mod store;
pub mod util;
mod verbosity;
mod version;

pub use context::ErrorContext;
pub use error::NixErrorCode;
pub use expr::{EvalState, EvalStateBuilder, Value, ValueType};
pub use store::{Store, StorePath};
pub use verbosity::NixVerbosity;
pub use version::NixVersion;

pub use nixide_sys as sys;

/// Sets the verbosity level
///
/// # Arguments
///
/// * `context` - additional error context, used as an output
/// * `level` - verbosity level
pub fn set_verbosity() {
    // nix_err nix_set_verbosity(nix_c_context * context, nix_verbosity level);
    // XXX: TODO: (implement Context first)
}
