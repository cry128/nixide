use std::env;
use std::path::PathBuf;
use std::process::Command;

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

    // Tell cargo to tell rustc to link the system shared library
    println!("cargo:rustc-link-lib=bz2");

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

    // Add all pkg-config include paths and GCC's include path to bindgen
    let mut args = Vec::new();
    for nix_lib in libs {
      let lib = pkg_config::probe_library(nix_lib)
        .expect(&format!("Unable to find .pc file for {}", nix_lib));
    
      for include_path in lib.include_paths {
        args.push(format!("-I{}", incloude_path.display()));
        // builder = builder.clang_arg(format!("-I{}", include_path.display()));
      }
      for link_file in lib.link_files {
        println!("cargo:rustc-link-lib={}", link_file.display());
      }
    }

    let lib_args = libs.map(|name| {
      let lib = pkg_config::probe_library(name)
        .expect(&format!("Unable to find .pc file for {}", nix_lib));

      for p in lib.link_files {
        println!("cargo:rustc-link-lib={}", p.display());
      }
    
      lib.include_paths.map(|p| format!("-I{}", p.display()))
    }).flatten();
    
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
        .rustfmt_configuration_file(std::fs::canonicalize(".rustfmt.toml").ok());
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
