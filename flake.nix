{
  description = "A basic devshell for developing starify.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        tailwindcss = pkgs.nodePackages.tailwindcss.overrideAttrs
          (oa: {
            plugins = [
              pkgs.nodePackages."@tailwindcss/aspect-ratio"
              pkgs.nodePackages."@tailwindcss/forms"
              pkgs.nodePackages."@tailwindcss/line-clamp"
              pkgs.nodePackages."@tailwindcss/typography"
            ];
          });
        
        rustToolchain =
          pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        name = (craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; }).pname;        
        src = with pkgs.lib;
          sources.cleanSourceWith {
            src = craneLib.path ./.;
            filter = path: type:
              (hasSuffix ".html" path) || (hasSuffix ".css" path) || (hasSuffix ".env" path)
              || (hasInfix "/assets/" path)
              || (hasSuffix ".js" path)
              || (hasSuffix ".json" path)
              || (craneLib.filterCargoSources path type);
          };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          cargo-leptos
          leptosfmt
          tailwindcss
          binaryen
          nodejs_latest
          dgraph
        ];

        commonArgs = { inherit src nativeBuildInputs; };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        bin = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          buildPhaseCargoCommand = "cargo leptos build --release -vvv";
          cargoTestCommand = "cargo leptos test --release -vvv";
          cargoExtraArgs = "";
          installPhaseCommand = ''
            mkdir -p $out/bin
            cp target/release/${name} $out/bin/
          '';
        });
      in {
        packages = {
          inherit bin;
          default = bin;
        };

        devShells.default = pkgs.mkShell {
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          inherit nativeBuildInputs;
        };
      });
}
