{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixpkgs-unstable";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
  };
  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs =
            with pkgs;
            [
              # Core build tools
              binutils
              dosfstools
              qemu

              (writeShellScriptBin "x" ''
                cargo xt $1 $2 $3 $4 $5 $6 $7 $8 $9 $10
              '')
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              # macOS-specific tools (hdiutil is built-in, no need to add)
              container
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
              # Linux-specific tools
              util-linux # for losetup on Linux (no-op on macOS)
              mount
              umount
            ];

          shellHook = ''
            echo -e "\n\033[1;32mshion development environment loaded"
            echo -e "Available tools:"
            echo -e "  - qemu-system-aarch64: $(which qemu-system-aarch64 2>/dev/null || echo 'not found')"
            echo -e "  - binutils: $(which readelf 2>/dev/null || echo 'not found')"
            echo -e "Platform: ${if pkgs.stdenv.isDarwin then "macOS" else "Linux"}\033[0m\n"
          '';
        };
      }
    );
}
