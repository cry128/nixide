use serial_test::serial;

use super::{Store, StorePath};
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrappers::AsInnerPtr as _;

#[test]
#[serial]
fn test_store_opening() {
    let mut ctx = ErrorContext::new();
    unsafe {
        sys::nix_libutil_init(ctx.as_ptr());
        ctx.pop()
            .expect("nix_libutil_init failed with bad ErrorContext");
        sys::nix_libstore_init(ctx.as_ptr());
        ctx.pop()
            .expect("nix_libstore_init failed with bad ErrorContext");
        sys::nix_libexpr_init(ctx.as_ptr());
        ctx.pop()
            .expect("nix_libexpr_init failed with bad ErrorContext");
    };

    let _store = Store::open(None).expect("Failed to open store");
}

#[test]
#[serial]
fn test_store_path_parse() {
    let mut ctx = ErrorContext::new();
    unsafe {
        sys::nix_libutil_init(ctx.as_ptr());
        ctx.pop()
            .expect("nix_libutil_init failed with bad ErrorContext");
        sys::nix_libstore_init(ctx.as_ptr());
        ctx.pop()
            .expect("nix_libstore_init failed with bad ErrorContext");
        sys::nix_libexpr_init(ctx.as_ptr());
        ctx.pop()
            .expect("nix_libexpr_init failed with bad ErrorContext");
    };

    let store = Store::open(None).expect("Failed to open store");

    // Try parsing a well-formed store path
    let result = StorePath::fake_path(&store);
    result.expect("idk hopefully this fails");
}

#[test]
#[serial]
fn test_store_path_clone() {
    let mut ctx = ErrorContext::new();
    unsafe {
        sys::nix_libutil_init(ctx.as_ptr());
        ctx.pop()
            .expect("nix_libutil_init failed with bad ErrorContext");
        sys::nix_libstore_init(ctx.as_ptr());
        ctx.pop()
            .expect("nix_libstore_init failed with bad ErrorContext");
        sys::nix_libexpr_init(ctx.as_ptr());
        ctx.pop()
            .expect("nix_libexpr_init failed with bad ErrorContext");
    };

    let store = Store::open(None).expect("Failed to open store");

    // Try to get a valid store path by parsing
    let path = StorePath::fake_path(&store).expect("Failed to create `StorePath::fake_path`");
    let cloned = path.clone();

    // Assert that the cloned path has the same name as the original
    let original_name = path.name().expect("Failed to get original path name");
    let cloned_name = cloned.name().expect("Failed to get cloned path name");

    assert_eq!(
        original_name, cloned_name,
        "Cloned path should have the same name as original"
    );
}

// Note: test_realize is not included because it requires a valid store path
// to realize, which we can't guarantee in a unit test. Integration tests
// would be more appropriate for testing realize() with actual derivations.
