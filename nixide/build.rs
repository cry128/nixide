use std::env;

fn main() {
    // allow the `cfg(nightly)` attribute
    println!("cargo::rustc-check-cfg=cfg(nightly)");

    // NOTE: This allows nixide to conditionally compile based
    // NOTE: whether a user has access to nightly features.
    if let Ok(toolchain) = env::var("RUSTUP_TOOLCHAIN")
        && toolchain.contains("nightly")
    {
        // enable the `nightly` flag
        println!("cargo::rustc-cfg=nightly");
    }
}
