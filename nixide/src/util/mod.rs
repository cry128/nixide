#[macro_use]
pub mod panic;
pub(crate) mod bindings;
mod cchar_nix_ext;
pub mod wrappers;

pub use cchar_nix_ext::CCharPtrNixExt;
pub(crate) use panic::*;

use crate::NixideError;

pub trait AsErr<T> {
    fn as_err(self) -> Result<(), T>;
}

impl AsErr<NixideError> for Option<NixideError> {
    fn as_err(self) -> Result<(), NixideError> {
        match self {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}
