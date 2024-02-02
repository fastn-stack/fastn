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

          buildInputs = with pkgs; lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];
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
          name = "fastn-shell";
          nativeBuildInputs = with pkgs; [ toolchain pkg-config openssl.dev postgresql_14 rust-analyzer diesel-cli ];

          shellHook = ''
            export PATH="$PATH:$HOME/.cargo/bin"
          '';
        };

        formatter = pkgs.nixpkgs-fmt;
      }
    );
}

