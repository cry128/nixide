//! # nixide-sys
//!
//! Unsafe direct FFI bindings to libnix C API.
//!
//! ## Safety
//!
//! These bindings are generated automatically and map directly to the C API.
//! They are unsafe to use directly. Prefer using the high-level safe API in the
//! parent crate unless you know what you're doing.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(rustdoc::bare_urls)]
#![allow(rustdoc::invalid_html_tags)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
