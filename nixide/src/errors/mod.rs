#[macro_use]
mod error;
mod context;
mod nix_error;

pub(crate) use context::ErrorContext;
pub(crate) use error::{new_nixide_error, retrace_nixide_error};
pub use error::{NixideError, NixideResult};
pub use nix_error::NixError;
