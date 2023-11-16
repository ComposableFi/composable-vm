{
  description = "Description for the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        # To import a flake module
        # 1. Add foo to inputs
        # 2. Add foo as a parameter to the outputs function
        # 3. Add here: foo.flakeModule

      ];
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem = { config, self', inputs', pkgs, system, ... }:

        let
          rust = (self.inputs.crane.mkLib pkgs).overrideToolchain
            (pkgs.rust-bin.stable."1.73.0".default.override {
              targets = [ "wasm32-unknown-unknown" ];
            });
        in
        {
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
                pkgs.virtualenv
                pkgs.conda
                pkgs.pyo3-pack
                rust.cargo
                rust.rustc
              ];
            };
          # Equivalent to  inputs'.nixpkgs.legacyPackages.hello;
          packages.default = pkgs.hello;
        };
      flake = {
        # The usual flake attributes can be defined here, including system-
        # agnostic ones like nixosModule and system-enumerating ones, although
        # those are more easily expressed in perSystem.

      };
    };
}
