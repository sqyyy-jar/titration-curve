{
  inputs = {
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = {
    self,
    fenix,
    flake-utils,
    naersk,
    nixpkgs,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        toolchain = with fenix.packages.${system};
          combine [
            minimal.rustc
            minimal.cargo
            targets.wasm32-unknown-unknown.latest.rust-std
          ];

        naersk' = naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        };

        naerskBuildPackage = target: args:
          naersk'.buildPackage (
            args
            // {CARGO_BUILD_TARGET = target;}
          );
      in rec {
        # For `nix build` & `nix run`:
        defaultPackage = packages.wasm32-unknown-unknown;

        packages.wasm32-unknown-unknown = naerskBuildPackage "wasm32-unknown-unknown" {
          src = ./.;
        };

        # For `nix develop`:
        devShell = pkgs.mkShell rec {
          inputsFrom = with packages; [wasm32-unknown-unknown];

          buildInputs = with pkgs; [pkg-config glib cairo gtk2 libsoup_3 webkitgtk_4_1 openssl];

          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
        };
      }
    );
}
