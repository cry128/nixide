mod attrs;
mod bool;
mod external;
mod float;
mod function;
mod int;
mod list;
mod null;
mod path;
mod string;
mod thunk;

pub use bool::NixBool;
pub use float::NixFloat;
pub use int::NixInt;
pub use string::NixString;

use std::fmt::{Debug, Display};
use std::ptr::NonNull;

use crate::sys;
use crate::util::wrappers::AsInnerPtr;

pub trait NixValue: Drop + Display + Debug + AsInnerPtr<sys::nix_value> {
    /// TODO
    fn id(&self) -> sys::ValueType;

    /// TODO
    fn new(inner: NonNull<sys::nix_value>) -> Self;
}
