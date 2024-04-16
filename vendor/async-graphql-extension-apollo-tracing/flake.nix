{
  description = "Dev env";

  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs @ {
    flake-parts,
    nixpkgs,
    flake-utils,
    crane,
    rust-overlay,
    ...
  }: let
    inherit (nixpkgs.lib) optional concatStringsSep;
    systems = flake-utils.lib.system;
    flake = flake-utils.lib.eachDefaultSystem (system: let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      aarch64DarwinExternalCargoCrates = concatStringsSep " " ["cargo-instruments@0.4.8" "cargo-about@0.6.1"];
      toolchain = pkgs.rust-bin.nightly.latest.default
;

      defaultShellConf = {
        buildInputs = [
          toolchain
        ];

        nativeBuildInputs = with pkgs;
          [
            protobuf
            mold
          ]
          ++ optional (system == systems.aarch64-darwin) [
            # cargo-binstall
            darwin.apple_sdk.frameworks.Foundation
            darwin.apple_sdk.frameworks.CoreFoundation
            darwin.apple_sdk.frameworks.CoreServices
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ]
          ++ optional (system != systems.aarch64-darwin) [
            cargo-about
          ];

        shellHook = ''
          # project_root="$(git rev-parse --show-toplevel 2>/dev/null || jj workspace root 2>/dev/null)"
          # export CARGO_INSTALL_ROOT="./.cargo";
          # if [[ "${system}" == "aarch64-darwin" ]]; then
          #  cargo binstall --no-confirm --no-symlinks --quiet ${aarch64DarwinExternalCargoCrates}
          # fi
        '';
      };
    in {
      devShells.default = pkgs.mkShell defaultShellConf;
    });
  in
    flake-parts.lib.mkFlake {inherit inputs;} {
      inherit flake;

      systems = flake-utils.lib.defaultSystems;

      perSystem = {
        config,
        system,
        ...
      }: {
        _module.args = {
          inherit crane;
          pkgs = import nixpkgs {
            inherit system;
            overlays = [(import rust-overlay)];
          };
        };
      };
    };
}
