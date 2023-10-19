{ pkgs ? import <nixpkgs> { } }:
let manifest = (pkgs.lib.importTOML ./fastn/Cargo.toml).package;
in
pkgs.rustPlatform.buildRustPackage rec {
  pname = manifest.name;
  version = manifest.version;
  cargoLock = {
    lockFile = ./Cargo.lock;
    # WTF? https://artemis.sh/2023/07/08/nix-rust-project-with-git-dependencies.html
    outputHashes = {
        "deadpool-0.9.5" = "sha256-4M2+nVVG/w0gnHkxTWVnfvy5HegW9A+nlWAkMltapeI=";
        "dioxus-core-0.3.2" = "sha256-jOVkqWPcGa/GGeZiQji7JbD2YF+qrXC9AZGozZg47+c=";
        "fbt-lib-0.1.18" = "sha256-xzhApWSVsVelii0R8vfB60kj0gA87MRTEplmX+UT96A=";
        "ftd-0.2.0" = "sha256-iHWR5KMgmo1QfLPc8ZKS4NvshXEg/OJw7c7fy3bs8v0=";
    };
  };
  src = pkgs.lib.cleanSource ./.;
}
