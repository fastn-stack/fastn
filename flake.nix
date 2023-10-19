{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";

    naersk.url = "github:nix-community/naersk";

    nixpkgs-mozilla = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };

    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  };

  outputs = { self, flake-utils, nixpkgs, nixpkgs-mozilla, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;

          overlays = [
            (import nixpkgs-mozilla)
          ];
        };

        toolchain = (pkgs.rustChannelOf {
          rustToolchain = ./rust-toolchain;
          sha256 = "sha256-Q9UgzzvxLi4x9aWUJTn+/5EXekC98ODRU1TwhUs9RnY=";
        }).rust;

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };

        fastnp = naersk'.buildPackage {
          name = "fastn";
          version = "0.3.0";
          src = ./.;

          nativeBuildInputs = with pkgs; [ pkg-config openssl.dev ];
        };
      in
      rec {
        # For `nix build` & `nix run`:
        defaultPackage = fastnp;

        packages = {
          fastn = fastnp;
        };

        # nix develop
        devShell = pkgs.mkShell {
          nativeBuildInputs = [ toolchain pkgs.pkg-config pkgs.openssl.dev ];
        };

        formatter = pkgs.nixpkgs-fmt;
      }
    );
}

