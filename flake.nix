{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";

    naersk.url = "github:nix-community/naersk";

    rust-overlay.url = "github:oxalica/rust-overlay";

    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, nixpkgs, rust-overlay, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;

          overlays = [
            (import rust-overlay)
          ];
        };

        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain;

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        fastn = naersk'.buildPackage {
          name = "fastn";
          version = cargoToml.workspace.package.version;
          src = pkgs.lib.cleanSource ./.;

          nativeBuildInputs = with pkgs; [
            pkg-config
            openssl.dev
          ] ++ lib.optionals stdenv.isDarwin [ xcbuild ];

          buildInput = with pkgs; lib.optionals stdenv.isDarwin [ darwin.Security ];
        };
      in
      rec {
        # For `nix build` & `nix run`:
        defaultPackage = fastn;

        packages = {
          inherit fastn;
        };

        # nix develop
        devShell = pkgs.mkShell {
          nativeBuildInputs = [ toolchain pkgs.pkg-config pkgs.openssl.dev ];
        };

        formatter = pkgs.nixpkgs-fmt;
      }
    );
}

