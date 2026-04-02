{
  description = "rust wrapper for libnix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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
          nixForBindings = pkgs.nixVersions.nix_2_34;
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
          nixForBindings = pkgs.nixVersions.nix_2_34;
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
