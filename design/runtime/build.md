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