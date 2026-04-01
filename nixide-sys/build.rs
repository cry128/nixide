use std::env;
use std::fs;
use std::path::PathBuf;

use bindgen::RustEdition;
use bindgen::callbacks::{ItemKind, ParseCallbacks};
use heck::ToSnekCase;
use heck::ToUpperCamelCase;
use itertools::Itertools;

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

/// Bindfmt is just the name im giving to the callbacks
/// that handle renaming C/C++ tokens.
#[derive(Debug)]
struct BindfmtCallbacks;

#[inline]
fn strip_variant_prefix(
    prefix: &'static str,
    enum_name: &str,
    variant: &str,
) -> Result<String, String> {
    variant
        .strip_prefix(prefix)
        .map(str::to_owned)
        .ok_or(format!(
            "[bindfmt] enum {enum_name} expected prefix \"{prefix}\" but got {}",
            &variant
        ))
}

impl ParseCallbacks for BindfmtCallbacks {
    fn enum_variant_name(
        &self,
        _enum_name: Option<&str>,
        _original_variant_name: &str,
        _variant_value: bindgen::callbacks::EnumVariantValue,
    ) -> Option<String> {
        let variant = _original_variant_name.to_upper_camel_case();

        _enum_name.map(|enum_name| match enum_name.to_upper_camel_case().as_ref() {
            "NixVerbosity" => strip_variant_prefix("NixLvl", enum_name, &variant).unwrap(),
            "NixErr" => strip_variant_prefix("NixErr", enum_name, &variant)
                .or_else(|_| strip_variant_prefix("Nix", enum_name, &variant))
                .unwrap(),
            "ValueType" => strip_variant_prefix("NixType", enum_name, &variant).unwrap(),
            _ => variant,
        })
    }

    fn item_name(&self, _item_info: bindgen::callbacks::ItemInfo) -> Option<String> {
        Some(match _item_info.kind {
            ItemKind::Type => _item_info.name.to_upper_camel_case(),
            _ => _item_info.name.to_snek_case(),
        })
    }

    fn include_file(&self, _filename: &str) {
        eprintln!("[debug] including file: {}", _filename);
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
    let include_paths: Vec<PathBuf> = LIBS
        .iter()
        .map(|&name| {
            let lib = pkg_config::probe_library(name)
                .expect(&format!("Unable to find .pc file for {}", name));

            for p in lib.link_files {
                println!("cargo::rustc-link-lib={}", p.display());
            }

            lib.include_paths
        })
        .flatten()
        .unique()
        .collect();

    let clang_args: Vec<String> = vec!["-x", "c++", "-std=c++23"]
        .into_iter()
        .map(|s: &str| s.to_owned())
        .chain(include_paths.iter().map(|p| format!("-I{}", p.display())))
        .collect();

    dbg!(&clang_args);

    let mut builder = bindgen::Builder::default()
        .rust_edition(RustEdition::Edition2024)
        .clang_args(clang_args)
        // Add `doxygen_bindgen` callbacks
        .parse_callbacks(Box::new(DoxygenCallbacks))
        .parse_callbacks(Box::new(BindfmtCallbacks))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Format generated bindings with rustfmt
        .formatter(bindgen::Formatter::Rustfmt)
        .rustfmt_configuration_file(std::fs::canonicalize("rustfmt.toml").ok())
        .allowlist_file(r".*nix_api_[a-z]+\.h")
        // Wrap all unsafe operations in unsafe blocks
        .layout_tests(true)
        .use_core() // use ::core instead of ::std
        .ctypes_prefix("::core::ffi") // use ::core::ffi instead of ::std::os::raw
        .time_phases(true)
        .wrap_unsafe_ops(true)
        .trust_clang_mangling(true)
        .respect_cxx_access_specs(true)
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
        .translate_enum_integer_types(false)
        .size_t_is_usize(true)
        .use_distinct_char16_t(false)
        .generate_comments(false)
        .generate_cstr(true) // use &CStr instead of &[u8]
        .fit_macro_constants(true)
        .explicit_padding(true)
        .enable_cxx_namespaces()
        .represent_cxx_operators(true)
        .enable_function_attribute_detection()
        .raw_line("/** These bindings were auto-generated for the Nixide project (https://github.com/cry128/nixide) */");

    // Register the input headers we would like to generate bindings for
    builder = LIBS
        .iter()
        .map(|lib| {
            let path = format!("include/{}.h", lib.strip_suffix("-c").unwrap());
            assert!(fs::exists(&path).unwrap());
            // Invalidate the built crate if the binding headers change
            println!("cargo::rerun-if-changed={path}");
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
