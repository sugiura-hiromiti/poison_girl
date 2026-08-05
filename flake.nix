# TODO: cache戦略の確定とcacheの導入
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

          xtaskWrapper =
            name:
            craneLib.mkCargoDerivation {
              pnameSuffix = "-workspace-${name}";
              buildPhaseCargoCommand = "cargo run --locked -p poison_girl -q -- --locked ${name}";
            };
        in
        {

          formatter = pkgs.nixfmt;

          checks = {
            # TODO: 以下の５種類をxtaskの薄いwrapperとして定義する
            # build, clippy, test, fmt
            # 以下はnix flake check 独自のlintとして定義する
            # doc, udeps, audit
            workspaceBuild = xtaskWrapper "build";
            workspaceTest = xtaskWrapper "test";
            workspaceClippy = xtaskWrapper "clippy";
            workspaceFmt = xtaskWrapper "fmt";
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
