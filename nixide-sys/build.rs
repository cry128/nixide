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
    println!("cargo:rerun-if-changed=include/nix-util.h");
    println!("cargo:rerun-if-changed=include/nix-store.h");
    println!("cargo:rerun-if-changed=include/nix-expr.h");
    println!("cargo:rerun-if-changed=include/nix-fetchers.h");
    println!("cargo:rerun-if-changed=include/nix-flake.h");
    println!("cargo:rerun-if-changed=include/nix-main.h");

    let libs = [
        #[cfg(feature = "nix-util-c")]
        "nix-util-c",
        #[cfg(feature = "nix-store-c")]
        "nix-store-c",
        #[cfg(feature = "nix-expr-c")]
        "nix-expr-c",
        #[cfg(feature = "nix-fetchers-c")]
        "nix-fetchers-c",
        #[cfg(feature = "nix-flake-c")]
        "nix-flake-c",
        #[cfg(feature = "nix-main-c")]
        "nix-main-c",
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

    let mut builder = bindgen::Builder::default()
        .clang_args(lib_args)
        // Invalidate the built crate when an included header file changes
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Add `doxygen_bindgen` callbacks
        .parse_callbacks(Box::new(DoxygenCallbacks))
        // Format generated bindings with rustfmt
        .formatter(bindgen::Formatter::Rustfmt)
        .rustfmt_configuration_file(std::fs::canonicalize(".rustfmt.toml").ok());

    // The input headers we would like to generate bindings for
    #[cfg(feature = "nix-util-c")]
    {
        builder = builder.header("include/nix-util.h")
    }
    #[cfg(feature = "nix-store-c")]
    {
        builder = builder.header("include/nix-store.h")
    }
    #[cfg(feature = "nix-expr-c")]
    {
        builder = builder.header("include/nix-expr.h")
    }
    #[cfg(feature = "nix-fetchers-c")]
    {
        builder = builder.header("include/nix-fetchers.h")
    }
    #[cfg(feature = "nix-flake-c")]
    {
        builder = builder.header("include/nix-flake.h")
    }
    #[cfg(feature = "nix-main-c")]
    {
        builder = builder.header("include/nix-main.h")
    }

    let bindings = builder
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
