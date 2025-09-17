{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, flake-utils, nixpkgs, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;

          overlays = [ (import rust-overlay) ];
        };

        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      rec {
        # nix develop
        devShell = pkgs.mkShell {
          name = "fastn-shell";
          nativeBuildInputs = with pkgs; [
            toolchain
            pkg-config
            openssl.dev
            diesel-cli
            rust-analyzer-unwrapped
            git
          ] ++ lib.optionals stdenv.isDarwin [  ];

          shellHook = ''
            export PATH="$PATH:$HOME/.cargo/bin"
          '';

          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
        };

        formatter = pkgs.nixpkgs-fmt;
      }
    );
}

