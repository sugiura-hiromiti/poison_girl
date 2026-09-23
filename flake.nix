{
  description = "poison girl dev env";

  inputs = {
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs = {
        nixpkgs = {
          follows = "nixpkgs";
        };
      };
    };
    files = {
      url = "github:mightyiam/files";
      flake = false;
    };
    github-actions-nix = {
      url = "github:synapdeck/github-actions-nix";
      inputs = {
        nixpkgs = {
          follows = "nixpkgs";
        };
        flake-parts = {
          follows = "flake-parts";
        };
      };
    };
    advisory-db = {
      url = "github:RustSec/advisory-db";
      flake = false;
    };

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
      treefmt-nix,
      files,
      github-actions-nix,
      advisory-db,
      nixpkgs,
      flake-parts,
      systems,
      fenix,
      crane,
      self,
    }:

    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        treefmt-nix.flakeModule
        github-actions-nix.flakeModules.default
        ./nix/ci.nix
      ];
      systems = import systems;
      perSystem =
        {
          pkgs,
          system,
          ...
        }:
        let
          fx = fenix.packages.${system};
          rust = fx.latest;
          rustToolchain = fx.combine [
            rust.toolchain
            rust.rust-src
            # fx.targets.aarch64-unknown-none.latest.rust-std
            # fx.targets.aarch64-unknown-uefi.latest.rust-std
          ];
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          cargoVendorDir = craneLib.vendorMultipleCargoDeps {
            cargoLockList = [
              ./Cargo.lock
              "${rust.rust-src}/lib/rustlib/src/rust/library/Cargo.lock"
            ];
          };
          ciCargoDerivationWrapper =
            command: name:
            craneLib.mkCargoDerivation {
              src = ./.;
              cargoLock = ./Cargo.lock;
              inherit cargoVendorDir;
              cargoArtifacts = null;
              pnameSuffix = "-workspace-${name}";
              buildPhaseCargoCommand = command;
              doInstallCargoArtifacts = false;
            };
          xtaskWrapper =
            name: ciCargoDerivationWrapper "cargo run --locked -p poison_girl -q -- --locked ${name}" name;
        in
        {
          treefmt = {
            settings = {
              global = {
                excludes = [ "target/**" ];
              };
            };
            programs = {
              taplo = {
                enable = true;
              };
              nixfmt = {
                enable = true;
              };
              rustfmt = {
                enable = true;
                package = rustToolchain;
              };
            };
          };

          checks = {
            workspaceBuild = xtaskWrapper "build";
            workspaceTest = xtaskWrapper "test";
            workspaceClippy = xtaskWrapper "clippy --deny-warnings";
            workspaceDoc = xtaskWrapper "doc --no-deps --document-private-items";
            workspaceUdeps = xtaskWrapper "udeps";
            workspaceAudit = craneLib.cargoAudit {
              src = ./.;
              inherit advisory-db;
            };
          };

          devShells = {
            default = craneLib.devShell {
              buildInputs =
                with pkgs;
                [
                  nixfmt
                  rustToolchain
                  taplo
                  # Core build tools
                  # binutils
                  qemu
                  dprint
                  cargo-nextest
                  cargo-udeps
                  cargo-audit
                  nixd
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
