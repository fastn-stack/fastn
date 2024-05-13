{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, flake-utils, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;

          overlays = [ ];
        };
      in
      rec {
        # nix develop
        devShell = pkgs.mkShell {
          name = "fastn-shell";
          nativeBuildInputs = with pkgs; [
            rustc
            rustfmt
            clippy
            cargo
            pkg-config
            openssl.dev
            postgresql_14
            diesel-cli
            rust-analyzer
          ];

          shellHook = ''
            export PATH="$PATH:$HOME/.cargo/bin"
          '';
        };

        formatter = pkgs.nixpkgs-fmt;
      }
    );
}

