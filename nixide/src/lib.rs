// #![warn(missing_docs)]

pub extern crate libc;
pub extern crate nixide_sys as sys;

pub(crate) mod errors;
mod stdext;
pub(crate) mod util;
mod verbosity;
mod version;

#[cfg(feature = "expr")]
mod expr;
#[cfg(feature = "store")]
mod store;

#[cfg(feature = "flake")]
mod flake;

pub use errors::{NixError, NixideError, NixideResult};
pub use verbosity::NixVerbosity;
pub use version::NixVersion;

#[cfg(feature = "expr")]
pub use expr::{EvalState, EvalStateBuilder, Value, ValueType};
#[cfg(feature = "store")]
pub use store::{Store, StorePath};

use ctor::ctor;
use util::wrappers::AsInnerPtr as _;

pub(crate) static mut INIT_LIBUTIL_STATUS: Option<NixideResult<()>> = None;
#[cfg(feature = "store")]
pub(crate) static mut INIT_LIBSTORE_STATUS: Option<NixideResult<()>> = None;
#[cfg(feature = "expr")]
pub(crate) static mut INIT_LIBEXPR_STATUS: Option<NixideResult<()>> = None;

/// # Warning
///
/// > Rust's philosophy is that nothing happens before or after main and [ctor](https://github.com/mmastrac/rust-ctor)
/// > explicitly subverts that. The code that runs in `ctor` functions
/// > should be careful to limit itself to libc functions and code
/// > that does not rely on Rust's stdlib services.
/// >  - Excerpt from the [github:mmastrac/rust-ctor README.md](https://github.com/mmastrac/rust-ctor?tab=readme-ov-file#warnings)
#[ctor]
fn init_libutil() {
    unsafe {
        INIT_LIBUTIL_STATUS = Some(util::wrap::nix_fn!(|ctx: &errors::ErrorContext| unsafe {
            sys::nix_libutil_init(ctx.as_ptr());
        }));
    }
}

/// # TODO
/// **Only run this if the "store" feature flag was enabled**
///
/// # Warning
///
/// > Rust's philosophy is that nothing happens before or after main and [ctor](https://github.com/mmastrac/rust-ctor)
/// > explicitly subverts that. The code that runs in `ctor` functions
/// > should be careful to limit itself to libc functions and code
/// > that does not rely on Rust's stdlib services.
/// >  - Excerpt from the [github:mmastrac/rust-ctor README.md](https://github.com/mmastrac/rust-ctor?tab=readme-ov-file#warnings)
#[ctor]
#[cfg(feature = "store")]
fn init_libstore() {
    // XXX: TODO: how do I support `sys::nix_libstore_init_no_load_config(context)`?
    unsafe {
        INIT_LIBSTORE_STATUS = Some(util::wrap::nix_fn!(|ctx: &errors::ErrorContext| unsafe {
            sys::nix_libutil_init(ctx.as_ptr());
        }));
    }
}

/// # TODO
/// **Only run this if the "expr" feature flag was enabled**
//
/// # Warning
///
/// > Rust's philosophy is that nothing happens before or after main and [ctor](https://github.com/mmastrac/rust-ctor)
/// > explicitly subverts that. The code that runs in `ctor` functions
/// > should be careful to limit itself to libc functions and code
/// > that does not rely on Rust's stdlib services.
/// >  - Excerpt from the [github:mmastrac/rust-ctor README.md](https://github.com/mmastrac/rust-ctor?tab=readme-ov-file#warnings)
#[ctor]
#[cfg(feature = "expr")]
fn init_libexpr() {
    unsafe {
        INIT_LIBEXPR_STATUS = Some(util::wrap::nix_fn!(|ctx: &errors::ErrorContext| unsafe {
            sys::nix_libexpr_init(ctx.as_ptr());
        }));
    }
}
