use std::path::PathBuf;
use std::{env, fs};

use bindgen::callbacks::ParseCallbacks;

#[derive(Debug)]
struct DoxygenCallbacks;

impl ParseCallbacks for DoxygenCallbacks {
    fn process_comment(&self, comment: &str) -> Option<String> {
        match doxygen_bindgen::transform(comment) {
            Ok(res) => Some(res),
            Err(err) => {
                println!("cargo::warning=Problem processing doxygen comment: {comment}\n{err}");
                None
            },
        }
    }
}

const LIBS: &[&'static str] = &[
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

fn main() {
    // Invalidate the built crate if the binding headers change
    // println!("cargo::rerun-if-changed=include");

    let lib_args: Vec<String> = LIBS
        .iter()
        .map(|&name| {
            let lib = pkg_config::probe_library(name)
                .expect(&format!("Unable to find .pc file for {}", name));

            for p in lib.link_files {
                println!("cargo::rustc-link-lib={}", p.display());
            }

            lib.include_paths
                .into_iter()
                .map(|p| format!("-I{}", p.display()))
        })
        .flatten()
        .chain(vec!["-Wall".to_owned(), "-xc++".to_owned()])
        .collect();

    let mut builder = bindgen::Builder::default()
        // .clang_arg("") // libnix uses c++23
        .clang_args(lib_args)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Add `doxygen_bindgen` callbacks
        .parse_callbacks(Box::new(DoxygenCallbacks))
        // Format generated bindings with rustfmt
        .formatter(bindgen::Formatter::Rustfmt)
        .rustfmt_configuration_file(std::fs::canonicalize("rustfmt.toml").ok());

    // Register the input headers we would like to generate bindings for
    builder = LIBS
        .iter()
        .map(|lib| {
            let path = format!("include/{}.h", lib.strip_suffix("-c").unwrap());
            assert!(fs::exists(&path).unwrap());
            // Invalidate the built crate if the binding headers change
            // println!("cargo::rerun-if-changed={path}");
            path
        })
        .fold(builder, |builder, path| builder.header(path));

    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let bindings = builder.generate().expect("Unable to generate bindings");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
