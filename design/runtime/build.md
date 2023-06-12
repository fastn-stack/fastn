# Building For Browser

We have to build two wasm files, `doc.wasm` and `runtime.wasm`. `doc.wasm` is built using [the compilation 
process](compilation.md). This document describes how to create `runtime.wasm` file, and how to test it locally,
without `fastn` server running.

## Setup

We have to install wasm32 target.

```sh
rustup target add wasm32-unknown-unknown
```

You have to re-run this command when you upgrade the Rust version.

## Building `runtime.wasm`

From `fastn-runtime` folder, run the following command:

```sh
cargo build --target wasm32-unknown-unknown --no-default-features --features=browser
```

Attach `--release` flag to create smaller binaries.

```txt
-rwxr-xr-x@ 1 amitu  staff   4.3M Jun 11 19:11 ../target/wasm32-unknown-unknown/debug/fastn_runtime.wasm
-rwxr-xr-x@ 1 amitu  staff   2.1M Jun 11 19:10 ../target/wasm32-unknown-unknown/release/fastn_runtime.wasm
```

## Minimize using `wasm-opt`

```sh
wasm-opt -O3  ../target/wasm32-unknown-unknown/release/fastn_runtime.wasm  -o f.wasm
ls -lh f.wasm
-rw-r--r--@ 1 amitu  staff   1.8M Jun 12 07:18 f.wasm
```

Gzip:

```shell
gzip f.wasm
ls -lh f.wasm.gz
-rw-r--r--@ 1 amitu  staff   397K Jun 12 07:18 f.wasm.gz
```

## Enabling `lto`


```toml
[profile.release]
lto = true
```

With LTO enabled, the sizes are:

```txt
-rwxr-xr-x@ 1 amitu  staff   4.3M Jun 11 19:11 ../target/wasm32-unknown-unknown/debug/fastn_runtime.wasm
-rwxr-xr-x@ 1 amitu  staff   518K Jun 12 07:24 ../target/wasm32-unknown-unknown/release/fastn_runtime.wasm
-rw-r--r--@ 1 amitu  staff   417K Jun 12 07:25 f.wasm
-rw-r--r--@ 1 amitu  staff   108K Jun 12 07:26 f.wasm.gz
```

## After Stripping Debug Info

```toml
[profile.release]
lto = true
strip = true
```

```shell
-rwxr-xr-x@ 1 amitu  staff   4.3M Jun 11 19:11 ../target/wasm32-unknown-unknown/debug/fastn_runtime.wasm
-rwxr-xr-x@ 1 amitu  staff   400K Jun 12 07:57 ../target/wasm32-unknown-unknown/release/fastn_runtime.wasm
-rw-r--r--@ 1 amitu  staff   353K Jun 12 07:58 f.wasm
-rw-r--r--@ 1 amitu  staff    89K Jun 12 07:58 f.wasm.gz
```
