{
  description = "poison girl dev env";

  inputs = {
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

          formatter = pkgs.nixfmt;

          checks = {
            # TODO: 以下の５種類をxtaskの薄いwrapperとして定義する
            # build, clippy, test
            # 以下はnix flake check 独自のlintとして定義する
            # doc, udeps, audit
            workspaceBuild = xtaskWrapper "build";
            workspaceTest = xtaskWrapper "test";
            workspaceClippy = xtaskWrapper "clippy --deny-warnings";
            workspaceFmt = ciCargoDerivationWrapper "cargo fmt --all --check" "fmt";
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
