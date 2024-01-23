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

          overlays = [
            (final: prev: {
              postgresql-static = (prev.postgresql.overrideAttrs (old: { dontDisableStatic = true; })).override {
                # We need libpq, which does not need systemd,
                # and systemd doesn't currently build with musl.
                enableSystemd = false;
              };
            })
          ];
        };

        fastn = pkgs.pkgsStatic.rustPlatform.buildRustPackage {
          name = "fastn";
          version = "0.4.47";
          src = pkgs.lib.cleanSource ./.;
          doCheck = false;

          nativeBuildInputs = [ pkgs.pkgsStatic.pkg-config pkgs.postgresql-static ];

          PKG_CONFIG_PATH = "${pkgs.pkgsStatic.openssl.dev}/lib/pkgconfig";

          buildFeatures = [ "auth" ];

          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
        };

        # my-bin = pkgs.pkgsCross.mingwW64.rustPlatform.buildRustPackage {
        #   name = "fastn";
        #   version = "0.4.42";
        #   src = pkgs.lib.cleanSource ./.;
        #   cargoLock = {
        #     lockFile = ./Cargo.lock;
        #     allowBuiltinFetchGit = true;
        #   };

        #   target = "x86_64-pc-windows-gnu";
        # };
      in
      rec {
        # For `nix build` & `nix run`:
        defaultPackage = fastn;

        packages = {
          inherit fastn;
        };

        # nix develop
        devShell = pkgs.mkShell {
          name = "fastn-shell";
          nativeBuildInputs = with pkgs; [ pkg-config openssl.dev postgresql_14 rust-analyzer ];

          shellHook = ''
            export PATH="$PATH:$HOME/.cargo/bin"
          '';
        };

        formatter = pkgs.nixpkgs-fmt;
      }
    );
}

