{
  description = "wasm";

  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        name = "wasm-shell";
        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
        buildInputs = with pkgs; [
          diesel-cli
          toolchain
          rust-analyzer-unwrapped
        ];

        RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
      };

      formatter.${system} = pkgs.nixpkgs-fmt;
    };
}

