{ rustPlatform, stdenv, pkg-config, lib, windows, openssl }:
rustPlatform.buildRustPackage {
  name = "fastn";
  version = "0.4.47";
  src = lib.cleanSource ./.;
  doCheck = false;

  nativeBuildInputs = [ pkg-config ];

  buildInputs = lib.optional stdenv.targetPlatform.isWindows [
    windows.mingw_w64_pthreads
    windows.pthreads
  ];

  # https://docs.rs/pkg-config/latest/pkg_config/
  PKG_CONFIG_ALL_STATIC = "1";

  PKG_CONFIG_PATH = "${openssl.dev}/lib/pkgconfig";

  RUSTFLAGS = "-C target-feature=+crt-static";

  buildFeatures = [ "auth" ];

  cargoLock = {
    lockFile = ./Cargo.lock;
    allowBuiltinFetchGit = true;
  };
}

