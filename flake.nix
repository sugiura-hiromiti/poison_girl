{
  description = "mogok dev env";
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixpkgs-unstable";
    };
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
    };
    systems = {
      url = "github:nix-systems/default";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs = {
        nixpkgs = {
          follows = "nixpkgs";
        };
      };
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs =
    inputs@{
      nixpkgs,
      flake-parts,
      systems,
      fenix,
      crane,
      self,
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import systems;
      perSystem =
        {
          self,
          pkgs,
          lib,
          system,
          config,
          specialArgs,
          options,
          ...
        }:
        let
          fx = fenix.packages.${system};
          rust = fx.latest;
          # pkgs = import nixpkgs {
          #   inherit system;
          #   config = {
          #     allowUnfree = true;
          #   };
          # };
          craneLib = (crane.mkLib pkgs).overrideToolchain rust.toolchain;
          src = lib.cleanSourceWith {
            src = craneLib.path ./.;
          };
          commonArgs = {
            inherit src;
            strictDeps = true;
            cargoLock = ./Cargo.lock;
            cargoExtraArgs = "-p poison_girl --locked";
          };
          deps = craneLib.buildDepsOnly commonArgs;
          myWorkspace = craneLib.cargoBuild (
            commonArgs
            // {
              cargoArtifacts = deps;
            }
          );
        in
        {
          formatter = pkgs.nixfmt;
          packages = {
            default = myWorkspace;
          };
          checks = {
            build = myWorkspace;
            clippy = craneLib.cargoClippy (
              commonArgs
              // {
                cargoArtifacts = deps;
                cargoClippyExtraArgs = "--all-targets";
              }
            );
            test = craneLib.cargoNextest (
              commonArgs
              // {
                cargoArtifacts = deps;
                cargoNextestExtraArgs = "--no-tests pass";
                partitions = 1;
                partitionType = "count";
              }
            );
            doc = craneLib.cargoDoc (
              commonArgs
              // {
                cargoArtifacts = deps;
                cargoDocExtraArgs = "--no-deps --document-private-items";
              }
            );
            fmt = craneLib.cargoFmt {
              inherit src;
              cargoExtraArgs = "--all";
            };
          };
          devShells = {
            default = craneLib.devShell {
              buildInputs =
                with pkgs;
                [
                  rust.toolchain
                  taplo
                  # Core build tools
                  binutils
                  dosfstools
                  qemu
                  dprint
                  cargo-nextest
                  cargo-udeps
                  cargo-audit
                ]
                ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                ]
                ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
                  util-linux # for losetup on Linux (no-op on macOS)
                  mount
                  umount
                ];
              shellHook = "";
            };
          };
        };
    };
}
