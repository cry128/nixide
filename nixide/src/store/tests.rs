use serial_test::serial;

use super::{Store, StorePath};
use crate::init::LIBNIX_INIT_STATUS;

#[test]
#[serial]
fn test_store_opening() {
    assert!(unsafe { matches!(LIBNIX_INIT_STATUS, Some(Ok(_))) });

    let _store = Store::default().expect("Failed to open store");
}

#[test]
#[serial]
fn test_store_path_parse() {
    assert!(unsafe { matches!(LIBNIX_INIT_STATUS, Some(Ok(_))) });

    let store = Store::default().expect("Failed to open store");

    // Try parsing a well-formed store path
    let result = StorePath::fake_path(&store.borrow());
    result.expect("idk hopefully this fails");
}

#[test]
#[serial]
fn test_store_path_clone() {
    assert!(unsafe { matches!(LIBNIX_INIT_STATUS, Some(Ok(_))) });

    let store = Store::default().expect("Failed to open store");

    // Try to get a valid store path by parsing
    let path =
        StorePath::fake_path(&store.borrow()).expect("Failed to create `StorePath::fake_path`");
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
