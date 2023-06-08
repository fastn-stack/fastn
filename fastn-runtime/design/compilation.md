# Compilation

`fastn-runtime` crate uses `wasm` to render all ftd programs. The input ftd file is compiled into a `wasm` file and is
fed to `fastn_runtime`. This document describes how the compilation process works, and how each ftd construct is mapped
to corresponding `wasm` construct.