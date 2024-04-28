{ rustPlatform, stdenv, pkg-config, lib, windows, openssl }:
let
  fastnCargo = builtins.fromTOML (builtins.readFile ./fastn/Cargo.toml);
  version = fastnCargo.package.version;
in
rustPlatform.buildRustPackage {
  name = "fastn";
  inherit version;
  src = lib.cleanSource ./.;

  doCheck = false; # set this to true to run cargo test

  nativeBuildInputs = [ pkg-config ];

  # TODO:use deno_core. can't do this until we can build rusty_v8 with musl
  # libc or figure out static linking with glibc
  buildFeatures = [ "quickjs" "fifthtry" ];

  buildInputs = lib.optional stdenv.targetPlatform.isWindows [
    windows.mingw_w64_pthreads
    windows.pthreads
  ];

  # https://docs.rs/pkg-config/latest/pkg_config/
  PKG_CONFIG_ALL_STATIC = "1";

  PKG_CONFIG_PATH = "${openssl.dev}/lib/pkgconfig";

  RUSTFLAGS = "-C target-feature=+crt-static";

  cargoLock = {
    lockFile = ./Cargo.lock;
    allowBuiltinFetchGit = true;
  };
}

