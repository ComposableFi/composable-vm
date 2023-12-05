{
  description = "CVM and MANTIS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    cosmos = {
      url = "github:dzmitry-lahoda-forks/cosmos.nix/dz/17";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
    };
    nix-std.url = "github:chessai/nix-std";
    devour-flake = {
      url = "github:srid/devour-flake";
      flake = false;
    };
  };

  outputs =
    inputs @ { flake-parts
    , self
    , crane
    , devour-flake
    , cosmos
    , nix-std
    , ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
      ];

      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem =
        { config
        , self'
        , inputs'
        , pkgs
        , system
        , ...
        }:
        let
          craneLib = crane.lib.${system};
          rust-src = pkgs.lib.cleanSourceWith {
            filter =
              pkgs.nix-gitignore.gitignoreFilterPure
                (
                  name: type:
                    !(pkgs.lib.strings.hasSuffix ".nix" name)
                    || builtins.match ".*proto$" name != null
                    || craneLib.filterCargoSources name type
                ) [ ./.gitignore ]
                ./.;
            src = craneLib.path ./.;
          };
          devour-flake = pkgs.callPackage inputs.devour-flake { };
          rust-toolchain =
            pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rust =
            (self.inputs.crane.mkLib pkgs).overrideToolchain
              rust-toolchain;
          defaultRustPlatform = pkgs.makeRustPlatform {
            cargo = rust-toolchain;
            rustc = rust-toolchain;
          };

          makeCosmwasmContract = name: rust: std-config:
            (cosmos.lib
              {
                inherit pkgs; cosmwasm-check = inputs.cosmos.packages.${system}.cosmwasm-check;
              }
            ).buildCosmwasmContract
              ({
                pname = name;
                name = name;
                src = rust-src;
                rustPlatform = defaultRustPlatform;
                buildNoDefaultFeatures = true;
                buildFeatures = std-config;
                cargoLock = {
                  lockFile = "${rust-src}/Cargo.lock";
                  outputHashes = {
                    "fixed-hash-0.8.0" = "sha256-KvkVqJZ5kvkKWXTYgG7+Ksz8aLhGZ1BG5zkM44fVNT4=";
                    "ibc-app-transfer-0.48.1" = "sha256-KvkVqJZ5kvkKWXTYgG7+Ksz8aLhGZ1BG5zkM44fVNT4=";
                    "ibc-apps-more-0.1.0" = "sha256-KvkVqJZ5kvkKWXTYgG7+Ksz8aLhGZ1BG5zkM44fVNT4=";
                    "serde-cw-value-0.7.0" = "sha256-KvkVqJZ5kvkKWXTYgG7+Ksz8aLhGZ1BG5zkM44fVNT4=";
                  };
                };
              } // rust-attrs
              );

          rust-attrs = {
            doCheck = false;
            checkPhase = "true";
            cargoCheckCommand = "true";
            NIX_BUILD_FLAKE = "true";
            RUST_BACKTRACE = "full";
            CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_DEBUG = true;
            buildInputs = [ pkgs.protobuf ];
          };
          cw-cvm-gateway = makeCosmwasmContract "cw-cvm-gateway" rust [ "std" ];
          cw-cvm-executor = makeCosmwasmContract "cw-cvm-executor" rust [ "std" ];
          cw-mantis-order = makeCosmwasmContract "cw-mantis-order" rust [ "std" ];
          cosmwasm-contracts = pkgs.symlinkJoin {
            name = "cosmwasm-contracts";
            paths = [
              cw-cvm-executor
              cw-cvm-gateway
              cw-mantis-order
            ];
          };
          cosmwasm-json-schema-ts = pkgs.writeShellApplication {
            name = "cosmwasm-json-schema-ts";
            runtimeInputs = with pkgs; [
              rust
              nodejs
              nodePackages.npm
            ];
            text = ''
              echo "generating TypeScript types and client definitions from JSON schema of CosmWasm contracts"
              cd code/cvm
              npm install
              rm --recursive --force dist

              rm --recursive --force schema
              cargo run --bin order --package cw-mantis-order
              npm run build-cw-mantis-order

              rm --recursive --force schema
              cargo run --bin gateway --package xc-core
              npm run build-xc-core

              npm publish
            '';
          };
        in
        {
          _module.args.pkgs = import self.inputs.nixpkgs {
            inherit system;
            overlays = with self.inputs; [
              rust-overlay.overlays.default
            ];
          };
          devShells.default =
            let
              python-packages = ps: with ps; [ numpy cvxpy wheel virtualenv ];
              python = pkgs.python3.withPackages python-packages;
            in
            pkgs.mkShell {
              VIRTUALENV_PYTHON = "${python}/bin/python3.11";
              VIRTUAL_ENV = 1;
              nativeBuildInputs = [ python ];
              buildInputs = [
                python
                devour-flake
                pkgs.virtualenv
                pkgs.conda
                pkgs.pyo3-pack
                rust.cargo
                rust.rustc
                devour-flake
              ];
              shellHook = ''
                if [[ -f ./.env ]]; then
                  source ./.env
                fi
              '';
            };
          formatter = pkgs.alejandra;
          packages = rec {
            inherit cw-mantis-order cw-cvm-executor cw-cvm-gateway cosmwasm-contracts;
            mantis = rust.buildPackage (rust-attrs
              // {
              src = rust-src;
              pname = "mantis";
              name = "mantis";
              cargoBuildCommand = "cargo build --release --bin mantis";
            });
            default = mantis;
            ci = pkgs.writeShellApplication {
              name = "nix-build-all";
              runtimeInputs = [
                pkgs.nix
                devour-flake
              ];
              text = ''
                nix flake lock --no-update-lock-file
                devour-flake . "$@"
              '';
            };
          };
        };
    };
}
