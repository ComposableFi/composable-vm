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
      url = "github:informalsystems/cosmos.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
    };

    devour-flake = {
      url = "github:srid/devour-flake";
      flake = false;
    };
  };

  outputs = inputs @ {
    flake-parts,
    self,
    crane,
    devour-flake,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
      ];

      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin"];
      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: let
        craneLib = crane.lib.${system};
        rust-src = pkgs.lib.cleanSourceWith {
          filter =
            pkgs.nix-gitignore.gitignoreFilterPure
            (
              name: type:
                !(pkgs.lib.strings.hasSuffix ".nix" name)
                || builtins.match ".*proto$" name != null
                || builtins.match ".*txt$" name != null
                || craneLib.filterCargoSources name type
            ) [./.gitignore]
            ./.;
          src = craneLib.path ./.;
        };
        devour-flake = pkgs.callPackage inputs.devour-flake {};
        rust-toolchain =
          pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rust =
          (self.inputs.crane.mkLib pkgs).overrideToolchain
          rust-toolchain;
        makeCosmwasmContract = name: rust: std-config: let
          binaryName = "${builtins.replaceStrings ["-"] ["_"] name}.wasm";
          maxWasmSizeBytes = 819200;
          profile = "deployment";
        in
          rust.buildPackage (rust-attrs
            // {
              src = rust-src;
              pnameSuffix = "-${name}";
              nativeBuildInputs = [
                pkgs.binaryen
                self.inputs.cosmos.packages.${system}.cosmwasm-check
              ];
              pname = name;
              cargoBuildCommand = "cargo build --target wasm32-unknown-unknown --profile ${profile} --package ${name} ${std-config}";
              RUSTFLAGS = "-C link-arg=-s";
              installPhaseCommand = ''
                mkdir --parents $out/lib
                # from CosmWasm/rust-optimizer
                # --signext-lowering is needed to support blockchains runnning CosmWasm < 1.3. It can be removed eventually
                wasm-opt target/wasm32-unknown-unknown/${profile}/${binaryName} -o $out/lib/${binaryName} -Os --signext-lowering
                cosmwasm-check $out/lib/${binaryName}
                SIZE=$(stat --format=%s "$out/lib/${binaryName}")
                if [[ "$SIZE" -gt ${builtins.toString maxWasmSizeBytes} ]]; then
                  echo "Wasm file size is $SIZE, which is larger than the maximum allowed size of ${builtins.toString maxWasmSizeBytes} bytes."
                  echo "Either reduce size or increase maxWasmSizeBytes if you know what you are doing."
                  exit 1
                fi
              '';
            });

        rust-attrs = {
          doCheck = false;
          checkPhase = "true";
          cargoCheckCommand = "true";
          NIX_BUILD_FLAKE = "true";
          RUST_BACKTRACE = "full";
          CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_DEBUG = true;
          buildInputs = [pkgs.protobuf];
        };
        cw-cvm-gateway = makeCosmwasmContract "cw-cvm-gateway" rust "--no-default-features --features=std,json-schema,cosmos";
        cw-cvm-executor = makeCosmwasmContract "cw-cvm-executor" rust "--no-default-features --features=std,json-schema,cosmos";
        cw-mantis-order = makeCosmwasmContract "cw-mantis-order" rust "--no-default-features --features=std,json-schema";
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
      in {
        _module.args.pkgs = import self.inputs.nixpkgs {
          inherit system;
          overlays = with self.inputs; [
            rust-overlay.overlays.default
          ];
        };
        devShells.default = let
          python-packages = ps: with ps; [numpy cvxpy wheel virtualenv];
          python = pkgs.python3.withPackages python-packages;
        in
          pkgs.mkShell {
            VIRTUALENV_PYTHON = "${python}/bin/python3.11";
            VIRTUAL_ENV = 1;
            nativeBuildInputs = [python pkgs.cbc];
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
              nativeBuildInputs = [pkgs.cbc];
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
