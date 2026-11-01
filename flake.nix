{
  description = "ff - A lightweight fuzzy finder for the terminal";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      ...
    }:
    let
      pname = "ff";
      version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
      # rust-toolchain.toml is the single source of truth for the compiler version.
      projectRustVersion =
        (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml)).toolchain.channel;
      # Hard ceiling for the compiled binary: 1 MiB.
      maxBinarySize = 1048576;
      inherit (nixpkgs) lib;
      allSystems = [
        "x86_64-linux"
        "aarch64-linux"
        # x86_64-darwin was dropped by nixpkgs 26.11.
        "aarch64-darwin"
      ];
      overlays = [ (import rust-overlay) ];
      forAllSystems =
        f:
        lib.genAttrs allSystems (
          system:
          f {
            pkgs = import nixpkgs { inherit system overlays; };
          }
        );
    in
    {
      packages = forAllSystems (
        { pkgs }:
        let
          rustVersion = pkgs.rust-bin.stable.${projectRustVersion}.default;
        in
        {
          default = pkgs.callPackage ./default.nix {
            inherit
              pname
              pkgs
              rustVersion
              self
              version
              ;
          };
        }
      );
      devShells = forAllSystems (
        { pkgs }:
        let
          rustVersion = pkgs.rust-bin.stable.${projectRustVersion}.default;
        in
        {
          default = pkgs.callPackage ./shell.nix {
            inherit
              pname
              pkgs
              rustVersion
              self
              version
              ;
          };
        }
      );
      checks = forAllSystems (
        { pkgs }:
        let
          package = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
        in
        {
          # Enforces the 1 MiB binary budget. Run with: nix flake check
          binary-size = pkgs.runCommand "ff-binary-size" { } ''
            size=$(wc -c < ${package}/bin/${pname} | tr -d ' ')
            echo "ff binary: $size bytes (limit: ${toString maxBinarySize})"
            if [ "$size" -gt ${toString maxBinarySize} ]; then
              echo "error: binary exceeds the 1 MiB budget by $((size - ${toString maxBinarySize})) bytes" >&2
              exit 1
            fi
            echo "headroom: $(( ${toString maxBinarySize} - size )) bytes"
            touch $out
          '';
        }
      );
      hydraJobs."${pname}" = forAllSystems ({ pkgs }: self.packages.${pkgs.stdenv.system}.default);
      # nix fmt
      formatter = nixpkgs.lib.genAttrs allSystems (system: nixpkgs.legacyPackages.${system}.nixfmt-tree);
    };
}
