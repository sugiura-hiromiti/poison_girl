{
  description = "poison girl dev env";
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
          rustToolchain = fx.combine [
            rust.toolchain
            rust.rust-src
            fx.targets.aarch64-unknown-none.latest.rust-std
            fx.targets.aarch64-unknown-uefi.latest.rust-std
          ];
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          src = lib.cleanSourceWith {
            src = ./.;
            filter =
              path: type:
              (craneLib.filterCargoSources path type)
              || lib.hasSuffix "/crates/kernel/aarch64-unknown-none-elf.json" path
              || lib.hasSuffix "/crates/kernel/x86_64-unknown-none-elf.json" path
              || lib.hasInfix "/crates/kernel/resource/" path
              || lib.hasInfix "/crates/macro/status/impl/status_" path;
          };
          workspaceArgs = {
            inherit src;
            strictDeps = true;
            cargoLock = ./Cargo.lock;
            cargoExtraArgs = "--workspace --locked";
          };
          workspaceDeps = craneLib.buildDepsOnly workspaceArgs;
          workspaceBuild = craneLib.cargoBuild (
            workspaceArgs
            // {
              cargoArtifacts = workspaceDeps;
            }
          );
          mkCargoCheck =
            args@{ cargoExtraArgs, ... }:
            craneLib.mkCargoDerivation (
              builtins.removeAttrs args [ "cargoExtraArgs" ]
              // {
                cargoArtifacts = null;
                doInstallCargoArtifacts = false;
                pnameSuffix = "-check";
                buildPhaseCargoCommand = "cargoWithProfile check ${cargoExtraArgs}";
                installPhaseCommand = "mkdir -p $out";
              }
            );
          kernelAarch64Check = mkCargoCheck {
            inherit src;
            pname = "poison_girl-kernel-aarch64";
            version = "0.1.0";
            strictDeps = true;
            cargoLock = ./Cargo.lock;
            cargoExtraArgs = "-p poison_girl_kernel --locked --target aarch64-unknown-none";
          };
          loaderAarch64UefiCheck = mkCargoCheck {
            inherit src;
            pname = "poison_girl-loader-aarch64-uefi";
            version = "0.1.0";
            strictDeps = true;
            cargoLock = ./Cargo.lock;
            cargoExtraArgs = "-p poison_girl_loader --locked --target aarch64-unknown-uefi";
          };
          workspaceMetadata =
            pkgs.runCommand "poison_girl-workspace-metadata"
              {
                nativeBuildInputs = [ rustToolchain ];
              }
              ''
                export CARGO_HOME="$TMPDIR/cargo-home"
                export RUSTUP_HOME="$TMPDIR/rustup-home"
                cp -R ${src} source
                chmod -R u+w source
                cd source
                cargo metadata --locked --no-deps --format-version 1 > "$out"
              '';
        in
        {
          formatter = pkgs.nixfmt;
          packages = {
            default = workspaceBuild;
          };
          checks = {
            workspace_metadata = workspaceMetadata;
            workspace_build = workspaceBuild;
            workspace_clippy = craneLib.cargoClippy (
              workspaceArgs
              // {
                cargoArtifacts = workspaceDeps;
                cargoClippyExtraArgs = "--all-targets -- -D warnings";
              }
            );
            workspace_test = craneLib.cargoNextest (
              workspaceArgs
              // {
                cargoArtifacts = workspaceDeps;
                cargoNextestExtraArgs = "--no-tests pass";
                partitions = 1;
                partitionType = "count";
              }
            );
            workspace_doc = craneLib.cargoDoc (
              workspaceArgs
              // {
                cargoArtifacts = workspaceDeps;
                cargoDocExtraArgs = "--no-deps --document-private-items";
              }
            );
            workspace_fmt = craneLib.cargoFmt {
              inherit src;
              cargoExtraArgs = "--all";
            };
            kernel_aarch64_check = kernelAarch64Check;
            loader_aarch64_uefi_check = loaderAarch64UefiCheck;
          };
          devShells = {
            default = craneLib.devShell {
              buildInputs =
                with pkgs;
                [
                  rustToolchain
                  taplo
                  # Core build tools
                  # binutils
                  qemu
                  dprint
                  cargo-nextest
                  cargo-udeps
                  cargo-audit
                  nil
                ]
                ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                ]
                ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
                ];
              shellHook = "";
            };
          };
        };
    };
}
