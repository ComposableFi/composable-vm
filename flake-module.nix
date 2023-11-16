{ self, ... }: {
  perSystem = { config, self', inputs', pkgs, system, crane, ... }:
    let
      rust = (self.inputs.crane.mkLib pkgs).overrideToolchain
        (pkgs.rust-bin.stable."1.73.0".default.override {
          targets = [ "wasm32-unknown-unknown" ];
        });
    in
    {
      packages = rec { 

      };


    };
}

