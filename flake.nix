# flake.nix
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
  };

  outputs =
    inputs@{
      nixpkgs,
      flake-parts,
      systems,
      fenix,
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
          pkgs = import nixpkgs {
            inherit system;
            config = {
              allowUnfree = true;
            };
          };
        in
        {
          devShells = {
            default = pkgs.mkShell {
              buildInputs =
                with pkgs;
                [
                  rust.toolchain
                  taplo
                  # Core build tools
                  binutils
                  dosfstools
                  qemu
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
