use std::env;
use std::path::PathBuf;

use bindgen::callbacks::ParseCallbacks;

#[derive(Debug)]
struct DoxygenCallbacks;

impl ParseCallbacks for DoxygenCallbacks {
    fn process_comment(&self, comment: &str) -> Option<String> {
        match doxygen_bindgen::transform(comment) {
            Ok(res) => Some(res),
            Err(err) => {
                println!("cargo:warning=Problem processing doxygen comment: {comment}\n{err}");
                None
            }
        }
    }
}

fn main() {
    // Invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=include/wrapper.h");

    // Use pkg-config to find nix-store include and link paths
    // This NEEDS to be included, or otherwise `nix_api_store.h` cannot
    // be found.
    let libs = [
        "nix-main-c",
        "nix-expr-c",
        "nix-store-c",
        "nix-util-c",
        "nix-flake-c",
    ];

    let lib_args: Vec<String> = libs
        .iter()
        .map(|&name| {
            let lib = pkg_config::probe_library(name)
                .expect(&format!("Unable to find .pc file for {}", name));

            for p in lib.link_files {
                println!("cargo:rustc-link-lib={}", p.display());
            }

            lib.include_paths
                .into_iter()
                .map(|p| format!("-I{}", p.display()))
        })
        .flatten()
        .collect();

    let bindings = bindgen::Builder::default()
        .clang_args(lib_args)
        // The input header we would like to generate bindings for
        .header("include/wrapper.h")
        // Invalidate the built crate when an included header file changes
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Add `doxygen_bindgen` callbacks
        .parse_callbacks(Box::new(DoxygenCallbacks))
        // Format generated bindings with rustfmt
        .formatter(bindgen::Formatter::Rustfmt)
        .rustfmt_configuration_file(std::fs::canonicalize(".rustfmt.toml").ok())
        // Finish the builder and generate the bindings
        .generate()
        // Unwrap the Result and panic on failure
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
