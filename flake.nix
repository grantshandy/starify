{
  description = "A basic devshell for developing musiscope.";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            openssl
            pkg-config
            cacert
            (rust-bin.stable.latest.default.override {
              extensions= [ "rust-src" "rust-analyzer" ];
              targets = [ "wasm32-unknown-unknown" ];
            })
            cargo-leptos
            cargo-watch
            tailwindcss
          ];
        };
      }
    );
}
