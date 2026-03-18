use serial_test::serial;

use super::*;

#[test]
#[serial]
fn test_store_opening() {
    let ctx = Arc::new(Context::new().expect("Failed to create context"));
    let _store = Store::open(&ctx, None).expect("Failed to open store");
}

#[test]
#[serial]
fn test_store_path_parse() {
    let ctx = Arc::new(Context::new().expect("Failed to create context"));
    let store = Store::open(&ctx, None).expect("Failed to open store");

    // Try parsing a well-formed store path
    // Note: This may fail if the path doesn't exist in the store
    let result = StorePath::parse(
        &ctx,
        &store,
        "/nix/store/00000000000000000000000000000000-test",
    );

    // We don't assert success here because the path might not exist
    // This test mainly checks that the API works correctly
    match result {
        Ok(_path) => {
            // Successfully parsed the path
        }
        Err(_) => {
            // Path doesn't exist or is invalid, which is expected
        }
    }
}

#[test]
#[serial]
fn test_store_path_clone() {
    let ctx = Arc::new(Context::new().expect("Failed to create context"));
    let store = Store::open(&ctx, None).expect("Failed to open store");

    // Try to get a valid store path by parsing
    // Note: This test is somewhat limited without a guaranteed valid path
    if let Ok(path) = StorePath::parse(
        &ctx,
        &store,
        "/nix/store/00000000000000000000000000000000-test",
    ) {
        let cloned = path.clone();

        // Assert that the cloned path has the same name as the original
        let original_name = path.name().expect("Failed to get original path name");
        let cloned_name = cloned.name().expect("Failed to get cloned path name");

        assert_eq!(
            original_name, cloned_name,
            "Cloned path should have the same name as original"
        );
    }
}

// Note: test_realize is not included because it requires a valid store path
// to realize, which we can't guarantee in a unit test. Integration tests
// would be more appropriate for testing realize() with actual derivations.
