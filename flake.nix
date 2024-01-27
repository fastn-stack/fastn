{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";

    nixpkgs.url = "github:junjihashimoto/nixpkgs/feature/rust-dup";
  };

  outputs = { self, flake-utils, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;

          overlays = [ ];
        };


        fastn = pkgs.pkgsStatic.callPackage ./fastn.nix { };
        fastn-win = pkgs.pkgsStatic.pkgsCross.mingwW64.callPackage ./fastn.nix { };
      in
      rec {
        # For `nix build` & `nix run`:
        defaultPackage = fastn;

        packages = {
          inherit fastn;
          inherit fastn-win;
        };

        # nix develop
        devShell = pkgs.mkShell {
          name = "fastn-shell";
          nativeBuildInputs = with pkgs; [
            rustc
            cargo
            pkg-config
            openssl.dev
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

