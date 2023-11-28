{
  description = "A basic devshell for developing musiscope.";

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

        # SPOTIFY_CLIENT_ID = builtins.readFile ./.spotify_client_id;
        # SPOTIFY_CLIENT_SECRET = builtins.readFile ./.spotify_client_secret;
        
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
              || (hasSuffix "tailwind.config.js" path)
              || (craneLib.filterCargoSources path type);
          };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          cargo-leptos
          tailwindcss
          binaryen
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

        dockerImage = pkgs.dockerTools.buildImage {
          inherit name;
          tag = "latest";
          copyToRoot = [ bin ];
          config = {
            Cmd = [ "${bin}/bin/${name}" ];
          };
        };
      in {
        packages = {
          inherit bin dockerImage;
          default = bin;
        };

        devShells.default = pkgs.mkShell {
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          inherit nativeBuildInputs;

          inputsFrom = [ bin ];
          buildInputs = with pkgs; [ docker dive neofetch ];
        };
      });
}
