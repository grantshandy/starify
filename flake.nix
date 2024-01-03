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

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane, ... }:
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

        linkNodeModules = ''
          ln -s ${(pkgs.callPackage ./tailwindcss.nix {}).nodeDependencies}/lib/node_modules ./node_modules
        '';

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile (self + /rust-toolchain.toml);
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = with pkgs; lib.cleanSourceWith {
              src = self; # The original, unfiltered source
              filter = path: type:
                (lib.hasSuffix "tailwind.config.js" path) ||
                (lib.hasSuffix ".css" path) ||
                (lib.hasInfix "/assets/" path) ||
                (lib.hasInfix "/css/" path) ||
                # Default filter from crane (allow .rs files)
                (craneLib.filterCargoSources path type)
              ;
            };

        cargoToml = builtins.fromTOML (builtins.readFile (self + /Cargo.toml));

        inherit (cargoToml.package) name version;

        # Crane builder for cargo-leptos projects
        craneBuild = rec {
          args = {
            inherit src version name;

            pname = name;
            buildInputs = with pkgs; [
              cargo-leptos
              binaryen
              tailwindcss
            ];
          };
          cargoArtifacts = craneLib.buildDepsOnly args;
          buildArgs = args // {
            LEPTOS_SITE_ROOT = "target/site";

            inherit cargoArtifacts;
            buildPhaseCargoCommand = "ln -s ${(pkgs.callPackage ./tailwindcss.nix {}).nodeDependencies}/lib/node_modules ./node_modules && cargo leptos build --release -vvv";
            installPhaseCommand = ''
              mkdir -p $out/bin
              cp target/server-release/${name} $out/bin/
            '';
          };
          package = craneLib.buildPackage buildArgs;
        };
      in {
        packages = {
          default = craneBuild.package;
        };

        devShells.default = pkgs.mkShell {
          shellHook = linkNodeModules;

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
            cargo-leptos
            leptosfmt
            tailwindcss
            binaryen
          ];
        };
      });
}
