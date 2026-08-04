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
              || lib.hasSuffix "/crates/kernel/aarch64-sugiura_hiromiti-poison_girl-elf.json" path
              || lib.hasSuffix "/crates/kernel/x86_64-sugiura_hiromiti-poison_girl-elf.json" path
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
          xtaskCheck = craneLib.mkCargoDerivation (
            workspaceArgs
            // {
              cargoArtifacts = workspaceDeps;
              pname = "poison_girl";
              version = "0.1.0";
              pnameSuffix = "-xtask-check";
              nativeBuildInputs = [ pkgs.cargo-nextest ];
              doInstallCargoArtifacts = false;
              buildPhaseCargoCommand =
                "cargo run --locked --package poison_girl --bin xtask-check";
              installPhaseCommand = "mkdir -p $out";
            }
          );
        in
        {
          formatter = pkgs.nixfmt;
          packages = {
            default = workspaceBuild;
          };
          checks = {
            xtask = xtaskCheck;
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
