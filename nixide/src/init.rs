use ctor::ctor;

use super::NixideResult;
use super::errors::ErrorContext;
use super::util::wrap;
use super::util::wrappers::AsInnerPtr as _;

pub(crate) static mut LIBNIX_INIT_STATUS: Option<NixideResult<()>> = None;

/// Initialises the Nix `libutil` library's global state.
/// This function **should not be run directly!** `#[ctor]` ensures it runs at init-time.
///
/// # Note
///
/// `sys::nix_libexpr_init` internally runs `sys::nix_libutil_init` and `sys::nix_libstore_init`.
/// Hence this method isn't registered via `#[ctor]` if the `exprs` feature is enabled.
///
/// # Warning
///
/// > Rust's philosophy is that nothing happens before or after main and [ctor](https://github.com/mmastrac/rust-ctor)
/// > explicitly subverts that. The code that runs in `ctor` functions
/// > should be careful to limit itself to libc functions and code
/// > that does not rely on Rust's stdlib services.
/// >  - Excerpt from the [github:mmastrac/rust-ctor README.md](https://github.com/mmastrac/rust-ctor?tab=readme-ov-file#warnings)
#[ctor]
#[cfg(not(feature = "exprs"))]
pub(crate) fn init_libutil() {
    unsafe {
        if !matches!(LIBNIX_INIT_STATUS, Some(Err(_))) {
            LIBNIX_INIT_STATUS = Some(wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
                sys::nix_libutil_init(ctx.as_ptr());
            }));
        }
    }
}

/// Initialises the Nix `libstore` library's global state.
/// This function **should not be run directly!** `#[ctor]` ensures it runs at init-time.
///
/// # Note
///
/// `sys::nix_libexpr_init` internally runs `sys::nix_libutil_init` and `sys::nix_libstore_init`.
/// Hence this method isn't registered via `#[ctor]` if the `exprs` feature is enabled.
///
/// # Warning
///
/// > Rust's philosophy is that nothing happens before or after main and [ctor](https://github.com/mmastrac/rust-ctor)
/// > explicitly subverts that. The code that runs in `ctor` functions
/// > should be careful to limit itself to libc functions and code
/// > that does not rely on Rust's stdlib services.
/// >  - Excerpt from the [github:mmastrac/rust-ctor README.md](https://github.com/mmastrac/rust-ctor?tab=readme-ov-file#warnings)
#[ctor]
#[cfg(all(feature = "store", not(feature = "exprs")))]
pub(crate) fn init_libstore() {
    // XXX: TODO: how do I support `sys::nix_libstore_init_no_load_config(context)`?
    unsafe {
        if !matches!(LIBNIX_INIT_STATUS, Some(Err(_))) {
            LIBNIX_INIT_STATUS = Some(wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
                sys::nix_libstore_init(ctx.as_ptr());
            }));
        }
    }
}

/// Initialises the Nix `libexpr` library's global state.
/// This function **should not be run directly!** `#[ctor]` ensures it runs at init-time.
///
/// # Note
///
/// `sys::nix_libexpr_init` internally runs `sys::nix_libutil_init` and `sys::nix_libstore_init`.
///
/// # Warning
///
/// > Rust's philosophy is that nothing happens before or after main and [ctor](https://github.com/mmastrac/rust-ctor)
/// > explicitly subverts that. The code that runs in `ctor` functions
/// > should be careful to limit itself to libc functions and code
/// > that does not rely on Rust's stdlib services.
/// >  - Excerpt from the [github:mmastrac/rust-ctor README.md](https://github.com/mmastrac/rust-ctor?tab=readme-ov-file#warnings)
#[ctor]
#[cfg(feature = "exprs")]
pub(crate) fn init_libexpr() {
    unsafe {
        if !matches!(LIBNIX_INIT_STATUS, Some(Err(_))) {
            LIBNIX_INIT_STATUS = Some(wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
                sys::nix_libexpr_init(ctx.as_ptr());
            }));
        }
    }
}
