{
  description = "rust wrapper for libnix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    systems.url = "github:nix-systems/default";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    ...
  } @ inputs: let
    systems = import inputs.systems;

    mkPkgs = system: repo:
      import repo {
        inherit system;
        allowUnfree = false;
        allowBroken = false;
        overlays = builtins.attrValues self.overlays or {};
        # config.replaceStdenv = {pkgs}: with pkgs; llvmPackages_21.stdenv;
      };

    forAllSystems = f:
      nixpkgs.lib.genAttrs systems (system:
        f rec {
          inherit system;
          inherit (pkgs) lib;
          pkgs = mkPkgs system nixpkgs;
        });
  in {
    overlays = {
      default = self: super: {
        libclang = super.llvmPackages_21.libclang;
      };
      fenix = inputs.fenix.overlays.default;
    };

    devShells = forAllSystems (
      {
        pkgs,
        lib,
        ...
      }: {
        default = let
          nixForBindings = pkgs.nixVersions.nix_2_32;
          inherit (pkgs.rustc) llvmPackages;
        in
          pkgs.mkShell rec {
            name = "nixide";
            shell = "${pkgs.bash}/bin/bash";
            strictDeps = true;

            # packages we need at runtime
            packages = with pkgs; [
              rustc
              llvmPackages.lld
              llvmPackages.lldb
              # lldb

              cargo
              cargo-c
              cargo-llvm-cov
              cargo-nextest

              clang # DEBUG
              clang-tools # DEBUG

              libcxx

              rust-analyzer-unwrapped
              (rustfmt.override {asNightly = true;})
              clippy
              taplo
            ];

            # packages we need at build time
            nativeBuildInputs = with pkgs; [
              pkg-config
              glibc.dev
              nixForBindings.dev

              rustPlatform.bindgenHook
            ];

            # packages we link against
            buildInputs = with pkgs; [
              stdenv.cc

              nixForBindings
            ];

            # bindgen uses clang to generate bindings, but it doesn't know where to
            # find our stdenv cc's headers, so when it's gcc, we need to tell it.
            postConfigure = lib.optionalString pkgs.stdenv.cc.isGNU ''
              #!/usr/bin/env bash
              # REF: https://github.com/nixops4/nix-bindings-rust/blob/main/bindgen-gcc.sh
              # Rust bindgen uses Clang to generate bindings, but that means that it can't
              # find the "system" or compiler headers when the stdenv compiler is GCC.
              # This script tells it where to find them.

              echo "Extending BINDGEN_EXTRA_CLANG_ARGS with system include paths..." 2>&1
              BINDGEN_EXTRA_CLANG_ARGS="$${BINDGEN_EXTRA_CLANG_ARGS:-}"
              export BINDGEN_EXTRA_CLANG_ARGS
              include_paths=$(
                echo | $NIX_CC_UNWRAPPED -v -E -x c - 2>&1 \
                | awk '/#include <...> search starts here:/{flag=1;next} \
                      /End of search list./{flag=0} \
                      flag==1 {print $1}'
              )
              for path in $include_paths; do
                echo " - $path" 2>&1
                BINDGEN_EXTRA_CLANG_ARGS="$BINDGEN_EXTRA_CLANG_ARGS -I$path"
              done
            '';

            shellHook = postConfigure;

            env = let
              inherit (llvmPackages) llvm libclang;
            in {
              LD_LIBRARY_PATH = builtins.toString (lib.makeLibraryPath buildInputs);
              LIBCLANG_PATH = "${libclang.lib}/lib";

              RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
              BINDGEN_EXTRA_CLANG_ARGS = "--sysroot=${pkgs.glibc.dev}";

              # `cargo-llvm-cov` reads these environment variables to find these binaries,
              # which are needed to run the tests
              LLVM_COV = "${llvm}/bin/llvm-cov";
              LLVM_PROFDATA = "${llvm}/bin/llvm-profdata";
            };
          };

        nightly = let
          nixForBindings = pkgs.nixVersions.nix_2_32;
          inherit (pkgs.rustc) llvmPackages;
        in
          pkgs.mkShell rec {
            name = "nixide";
            shell = "${pkgs.bash}/bin/bash";
            strictDeps = true;

            # packages we need at runtime
            packages = with pkgs; [
              llvmPackages.lld
              lldb
              (pkgs.fenix.complete.withComponents [
                "cargo"
                "clippy"
                "rust-src"
                "rustc"
                "rustfmt"
              ])
              rust-analyzer-nightly

              # cargo-c
              # cargo-llvm-cov
              # cargo-nextest
            ];

            # packages we need at build time
            nativeBuildInputs = with pkgs; [
              pkg-config
              glibc.dev
              nixForBindings.dev

              rustPlatform.bindgenHook
            ];

            # packages we link against
            buildInputs = with pkgs; [
              stdenv.cc

              nixForBindings
            ];

            # bindgen uses clang to generate bindings, but it doesn't know where to
            # find our stdenv cc's headers, so when it's gcc, we need to tell it.
            postConfigure = lib.optionalString pkgs.stdenv.cc.isGNU ''
              #!/usr/bin/env bash
              # REF: https://github.com/nixops4/nix-bindings-rust/blob/main/bindgen-gcc.sh
              # Rust bindgen uses Clang to generate bindings, but that means that it can't
              # find the "system" or compiler headers when the stdenv compiler is GCC.
              # This script tells it where to find them.

              echo "Extending BINDGEN_EXTRA_CLANG_ARGS with system include paths..." 2>&1
              BINDGEN_EXTRA_CLANG_ARGS="$${BINDGEN_EXTRA_CLANG_ARGS:-}"
              export BINDGEN_EXTRA_CLANG_ARGS
              include_paths=$(
                echo | $NIX_CC_UNWRAPPED -v -E -x c - 2>&1 \
                | awk '/#include <...> search starts here:/{flag=1;next} \
                      /End of search list./{flag=0} \
                      flag==1 {print $1}'
              )
              for path in $include_paths; do
                echo " - $path" 2>&1
                BINDGEN_EXTRA_CLANG_ARGS="$BINDGEN_EXTRA_CLANG_ARGS -I$path"
              done
            '';

            shellHook = postConfigure;

            env = let
              inherit (llvmPackages) llvm libclang;
            in {
              LD_LIBRARY_PATH = builtins.toString (lib.makeLibraryPath buildInputs);
              LIBCLANG_PATH = "${libclang.lib}/lib";

              RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
              BINDGEN_EXTRA_CLANG_ARGS = "--sysroot=${pkgs.glibc.dev}";

              # `cargo-llvm-cov` reads these environment variables to find these binaries,
              # which are needed to run the tests
              LLVM_COV = "${llvm}/bin/llvm-cov";
              LLVM_PROFDATA = "${llvm}/bin/llvm-profdata";
            };
          };
      }
    );
  };
}
