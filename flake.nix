{
  description = "Wire on your TTYs just feels better!";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    systems.url = "github:nix-systems/default";

    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
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
      };

    forAllSystems = f:
      nixpkgs.lib.genAttrs systems (system:
        f rec {
          inherit system;
          inherit (pkgs) lib;
          pkgs = mkPkgs system nixpkgs;
        });
  in {
    overlays.default = self: super: {
      libclang = super.llvmPackages_21.libclang;
    };

    devShells = forAllSystems (
      {
        system,
        pkgs,
        lib,
        ...
      }: {
        default = pkgs.mkShell rec {
          shell = "${pkgs.bash}/bin/bash";
          strictDeps = true;

          packages = with pkgs; [
            cargo
            rustc
            inputs.fenix.packages.${system}.complete.rustfmt
          ];

          # packages we should be able to link against
          buildInputs = with pkgs; [
            # pipewire.dev
            # libxkbcommon
            # wayland
          ];

          # packages we run at build time / shellHook
          nativeBuildInputs = with pkgs; [
            pkg-config
            rustPlatform.bindgenHook
          ];

          LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${builtins.toString (lib.makeLibraryPath buildInputs)}";
        };
      }
    );
  };
}
