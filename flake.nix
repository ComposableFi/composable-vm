{
  description = "CVM and MANTIS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix2container = {
      url = "github:nlewo/nix2container";
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
    datamodel-code-generator-src = {
      url = "github:koxudaxi/datamodel-code-generator";
      flake = false;
    };
    poetry2nix = {
      url = "github:nix-community/poetry2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    networks = {
      url = "github:ComposableFi/networks";
    };

    cosmpy-src = {
      url = "github:fetchai/cosmpy";
      flake = false;
    };

    fastapi-cache-src = {
      url = "github:long2ice/fastapi-cache";
      flake = false;
    };

    scip = {
      url = github:dzmitry-lahoda-forks/scip/169747d9a7d5b01a44722ea7db2ed389443e7a57;
    };
    devenv.url = "github:cachix/devenv";
    strictly-typed-pandas-src = {
      url = "github:nanne-aben/strictly_typed_pandas";
      flake = false;
    };

    maturin-src = {
      url = "github:PyO3/maturin";
      flake = false;
    };

    pydantic-src = {
      url = "github:pydantic/pydantic/v2.5.3";
      flake = false;
    };

    scipy-src = {
      url = "github:scipy/scipy/v1.9.3";
      flake = false;
    };

    cvxpy-src = {
      url = "github:cvxpy/cvxpy/v1.3.2";
      flake = false;
    };

    devour-flake = {
      url = "github:srid/devour-flake";
      flake = false;
    };

    pyscipopt-src = {
      url = "github:scipopt/PySCIPOpt/v4.3.0";
      flake = false;
    };
  };

  outputs = inputs @ {
    flake-parts,
    self,
    crane,
    devour-flake,
    datamodel-code-generator-src,
    poetry2nix,
    cosmpy-src,
    nixpkgs,
    scip,
    pyscipopt-src,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.devenv.flakeModule
      ];
      # would be happy to support more, so solver engines are crazy heavy on some hardcode deps
      systems = ["x86_64-linux" "aarch64-darwin"];
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
          maxWasmSizeBytes = 819200 * 2;
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
        cw-cvm-outpost = makeCosmwasmContract "cw-cvm-outpost" rust "--no-default-features --features=std,json-schema,cosmos";
        cw-cvm-executor = makeCosmwasmContract "cw-cvm-executor" rust "--no-default-features --features=std,json-schema,cosmos";
        cw-mantis-order = makeCosmwasmContract "cw-mantis-order" rust "--no-default-features --features=std,json-schema";
        cosmwasm-contracts = pkgs.symlinkJoin {
          name = "cosmwasm-contracts";
          paths = [
            cw-cvm-executor
            cw-cvm-outpost
            cw-mantis-order
          ];
        };
        cosmwasm-json-schema-ts = pkgs.writeShellApplication {
          name = "cosmwasm-json-schema-ts";
          runtimeInputs = with pkgs; [
            rust.rustc
            rust.cargo
            nodejs
            nodePackages.npm
          ];
          text = ''
            echo "generating TypeScript types and client definitions from JSON schema of CosmWasm contracts"
            cd contracts/cosmwasm/
            npm install
            rm --recursive --force dist

            rm --recursive --force schema
            cargo run --bin order --package cw-mantis-order --features=std,json-schema
            npm run build-cw-mantis-order

            rm --recursive --force schema
            cargo run --bin outpost --package cvm-runtime --features=std,json-schema,cosmos,cosmwasm
            npm run build-cvm-runtime

            npm publish
          '';
        };
      in let
        datamodel-code-generator = mkPoetryApplication {
          projectDir = datamodel-code-generator-src;
          checkGroups = [];
        };

        cosmpy = pkgs.python3Packages.buildPythonPackage {
          name = "cosmpy";
          version = "0.9.1";
          format = "pyproject";

          src = cosmpy-src;

          nativeBuildInputs = [
            pkgs.python3Packages.poetry-core
          ];
        };

        # https://github.com/nanne-aben/strictly_typed_pandas/issues/140
        strictly-typed-pandas-latest = pkgs.python3Packages.buildPythonPackage {
          name = "strictly-typed-pandas";
          version = "0.0.1";
          format = "pyproject";

          src = inputs.strictly-typed-pandas-src;

          nativeBuildInputs = with pkgs.python3Packages; [
            poetry-core
            setuptools
            setuptools-git-versioning
          ];
        };

        scipy-latest = pkgs.python3Packages.buildPythonPackage {
          name = "scipy";
          version = "0.0.1";
          format = "pyproject";

          src = inputs.scipy-src;

          nativeBuildInputs = with pkgs.python3Packages; [
            poetry-core
            meson
            meson-python
            setuptools
            setuptools-git-versioning
            pkgs.pkg-config
          ];
        };

        pyscipopt-latest = pkgs.python3Packages.buildPythonPackage {
          name = "pyscipopt";
          version = "v4.3.0";
          format = "pyproject";

          src = inputs.pyscipopt-src;

          nativeBuildInputs = with pkgs.python3Packages; [
            setuptools
            pkgs.pkg-config
            inputs'.scip.packages.scip
            pkgs.python311Packages.cython
          ];
          buildInputs = with pkgs.python3Packages; [
            inputs'.scip.packages.scip
            cython
          ];
        };

        dep = name:
          builtins.head (pkgs.lib.lists.filter
            (x: pkgs.lib.strings.hasInfix name x.name)
            poetryDeps.poetryPackages);

        cvxpy-latest = pkgs.python3Packages.buildPythonPackage {
          name = "cvxpy";
          version = "1.3.2";
          format = "pyproject";

          src = inputs.cvxpy-src;

          nativeBuildInputs = with pkgs.python3Packages; [
            (dep "numpy")
            (dep "scipy")
            poetry-core
            setuptools
            setuptools-git-versioning
            pkgs.pkg-config
          ];
        };

        maturin-latest = pkgs.python3Packages.buildPythonPackage {
          name = "maturin";
          version = "0.0.1";
          format = "pyproject";

          src = inputs.maturin-src;

          cargoDeps = pkgs.rustPlatform.fetchCargoTarball {
            src = inputs.maturin-src;
            name = "maturin";
            version = "0.0.1";
            hash = "sha256-3zPG/6EQiXaDaxgNYIAsJ5A7MbbK2Gxj8NDpEukUit4=";
          };

          nativeBuildInputs = with pkgs.python3Packages; [
            poetry-core
            setuptools
            setuptools-rust
            setuptools-git-versioning
            pkgs.rustPlatform.cargoSetupHook
            pkgs.rustPlatform.maturinBuildHook
          ];
        };

        poetryDeps = mkPoetryPackages {
          projectDir = ./mantis;
        };

        override = overrides:
          overrides.withDefaults (self: super: {
            editables = super.editables.overridePythonAttrs (old: {
              buildInputs = old.buildInputs or [] ++ [self.python.pkgs.flit-core];
            });

            pydantic-extra-types = super.pydantic-extra-types.overridePythonAttrs (old: {
              buildInputs = old.buildInputs or [] ++ [self.python.pkgs.hatchling];
            });

            clarabel = super.pydantic-extra-types.overridePythonAttrs (old: {
              buildInputs = old.buildInputs or [] ++ [self.python.pkgs.maturin];
              nativeBuildInputs = old.buildInputs or [] ++ [self.python.pkgs.maturin];
            });

            pyscipopt = pyscipopt-latest;
            google = super.google.overridePythonAttrs (old: {
              buildInputs = old.buildInputs or [] ++ [self.python.pkgs.setuptools];
            });
            cylp = super.cylp.overridePythonAttrs (old: {
              buildInputs = old.buildInputs or [] ++ [self.python.pkgs.setuptools self.python.pkgs.wheel pkgs.cbc pkgs.pkg-config];
              nativeBuildInputs = old.nativeBuildInputs or [] ++ [self.python.pkgs.setuptools self.python.pkgs.wheel pkgs.cbc pkgs.pkg-config];
            });
            google-cloud = super.google-cloud.overridePythonAttrs (old: {
              buildInputs = old.buildInputs or [] ++ [self.python.pkgs.setuptools];
            });
            cvxpy = cvxpy-latest;
            cosmpy = cosmpy;

            maturin = maturin-latest;
            strictly-typed-pandas = strictly-typed-pandas-latest;
          });

        envShell = mkPoetryEnv {
          projectDir = ./mantis;
          overrides = override overrides;
        };
        mantis-blackbox-package = mkPoetryApplication {
          projectDir = ./mantis;
          overrides = override overrides;
        };

        mantis-blackbox = pkgs.writeShellApplication {
          name = "mantis-blackbox";
          runtimeInputs = [mantis-blackbox-package];
          text = ''
            # shellcheck disable=SC2068
            mantis-blackbox $@
          '';
        };

        inherit (poetry2nix.lib.mkPoetry2Nix {inherit pkgs;}) mkPoetryApplication mkPoetryPackages mkPoetryEnv overrides;
        env = {
          OSMOSIS_POOLS = "https://app.osmosis.zone/api/pools?page=1&limit=1000&min_liquidity=500000";
          ASTROPORT_POOLS = "https://app.astroport.fi/api/trpc/pools.getAll?input=%7B%22json%22%3A%7B%22chainId%22%3A%5B%22neutron-1%22%5D%7D%7D";
          SKIP_MONEY_SWAGGER = "https://api-swagger.skip.money/";
          SKIP_MONEY = "https://api.skip.money/";
        };
        cosmwasm-json-schema-py = let
        in
          pkgs.writeShellApplication {
            name = "cosmwasm-json-schema-py";
            runtimeInputs = with pkgs; [
              rust.cargo
              datamodel-code-generator
            ];
            text = ''
              RUST_BACKTRACE=1 cargo run --package cvm-runtime --bin outpost --features=cosmwasm,json-schema,cosmos,std

              datamodel-codegen  --input schema/raw/ --input-file-type jsonschema --output mantis/blackbox/cvm_runtime/  --disable-timestamp --target-python-version "3.10" --use-schema-description --output-model-type "pydantic_v2.BaseModel"

              curl "${env.OSMOSIS_POOLS}" | jq .pools > schema/osmosis_pools.json
              datamodel-codegen  --input schema/osmosis_pools.json --input-file-type json --output mantis/blackbox/osmosis_pools.py  --disable-timestamp --target-python-version "3.10" --use-schema-description --output-model-type "pydantic_v2.BaseModel"

              curl "${env.ASTROPORT_POOLS}" | jq .result.data > schema/neutron_pools.json
              datamodel-codegen  --input schema/neutron_pools.json --input-file-type json --output mantis/blackbox/neutron_pools.py  --disable-timestamp --target-python-version "3.10" --use-schema-description --output-model-type "pydantic_v2.BaseModel"

              curl "${env.SKIP_MONEY_SWAGGER}swagger.yml" > schema/skip_money_swagger.yml
              datamodel-codegen  --input schema/skip_money_swagger.yml --input-file-type openapi --output mantis/blackbox/skip_money.py  --disable-timestamp --target-python-version "3.10" --use-schema-description --output-model-type "pydantic_v2.BaseModel"

              curl --request GET --url https://api.skip.money/v1/info/chains --header 'accept: application/json' | jq . > schema/skip_money_chain.json
              curl --request GET --url https://api.skip.money/v1/fungible/assets --header 'accept: application/json' | jq . > schema/skip_money_assets.json
            '';
          };
        native-deps = [
          pkgs.cbc
          inputs'.scip.packages.scip
          pkgs.CoinMP
          pkgs.ipopt
          pkgs.or-tools
        ];
      in {
        _module.args.pkgs = import self.inputs.nixpkgs {
          inherit system;
          overlays = with self.inputs; [
            rust-overlay.overlays.default
          ];
        };
        devenv.shells.default = {
          env = {
            OSMOSIS_POOLS = env.OSMOSIS_POOLS;
            ASTROPORT_POOLS = env.ASTROPORT_POOLS;
            SKIP_MONEY = env.SKIP_MONEY;
            COMPOSABLE_COSMOS_GRPC = inputs.networks.lib.pica.mainnet.GRPC;
            CVM_ADDRESS = inputs.networks.lib.pica.mainnet.CVM_OUTPOST_CONTRACT_ADDRESS;
            LD_LIBRARY_PATH = pkgs.lib.strings.makeLibraryPath [
              pkgs.stdenv.cc.cc.lib
              pkgs.zlib
              pkgs.zlib.dev
              pkgs.zlib.out

              "${inputs'.scip.packages.scip}/lib"
            ];
          };

          packages =
            [
              pkgs.nix
              pkgs.zlib
              pkgs.zlib.dev
              pkgs.zlib.out
              devour-flake
              pkgs.conda
              pkgs.nodejs
              pkgs.nodePackages.npm
              pkgs.poetry
              pkgs.pyo3-pack
              pkgs.python3Packages.flit
              pkgs.python3Packages.flit-core
              pkgs.python3Packages.uvicorn
              pkgs.virtualenv
              pkgs.zlib
              pkgs.zlib.dev
              pkgs.zlib.out
              rust.cargo
              rust.rustc
              envShell
              devour-flake
            ]
            ++ native-deps;
          devcontainer.enable = true;
          enterShell = ''
            if [[ -f ./.env ]]; then
              source ./.env
            fi
            (
              cd mantis
              poetry install
            )
          '';
        };
        formatter = pkgs.alejandra;
        packages = rec {
          inherit
            cw-mantis-order
            cw-cvm-executor
            cw-cvm-outpost
            cosmwasm-contracts
            cosmwasm-json-schema-py
            datamodel-code-generator
            cosmwasm-json-schema-ts
            mantis-blackbox
            pyscipopt-latest
            ;
          all =
            pkgs.linkFarmFromDrvs "all"
            (with self'.packages; [
              cw-mantis-order
              cw-cvm-executor
              cw-cvm-outpost
              cosmwasm-contracts
              cosmwasm-json-schema-py
              datamodel-code-generator
              cosmwasm-json-schema-ts
              mantis-blackbox
            ]);
          mantis = rust.buildPackage (rust-attrs
            // {
              src = rust-src;
              pname = "mantis";
              name = "mantis";
              cargoBuildCommand = "cargo build --release --bin mantis";
              nativeBuildInputs = native-deps;
            });
          default = mantis-blackbox;
          ci = pkgs.writeShellApplication {
            name = "nix-build-all";
            runtimeInputs = [
              pkgs.nix
              devour-flake
            ];
            text = ''
              (
                cd mantis
                echo "running tests"
                nix develop --impure --command poetry run pytest
                nix develop --impure --command poetry check --lock
              )
              nix flake show --all-systems --json --no-write-lock-file
              nix flake lock --no-update-lock-file
              # devour-flake . "$@" --impure
              nix build .#all
            '';
          };
        };
      };
    };
}
