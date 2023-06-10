# Browser

[Back To Design Home](..).

`fastn_runtime` uses `WebAssembly` for executing functions defined in the ftd file, event handlers etc. 

## `doc.wasm`

Every ftd file is converted to a wasm file (refer [`compilation.md`](compilation.md) for details). In this document we 
will call the  file `doc.wasm` i.e. `doc.ftd` gets compiled into `doc.wasm`.

## `doc.json` and `doc.html`

The `doc.wasm` file is used to create a wasm instance, and a function named `main` that is exported by `doc.wasm` is
called. The `main` creates all the global variables, and the DOM. The variable data, the event handlers and the DOM,
are captured in two files, `doc.json` and `doc.html`. 

`doc.html` contains fully rendered static version of `doc.ftd`. It will look exactly how it should but event handlers 
would not work. 

## `linker.js`

Checkout [`linker.md`](linking.md) for details. For now we are going ahead with the Approach a discussed there.

## `runtime.wasm`

The runtime itself is written in Rust and gets compiled into a file `runtime.wasm`. 

### Versioning

The `runtime.wasm` only changes when `fastn` itself changes, so we can serve the `runtime.wasm` from a global CDN, and
the actual URL for `runtime.wasm` can be versioned, like `runtime-<hash>.wasm`.

## Server Side Rendering

Checkout the [`ssr.md`](ssr.md) for a discussion on server side render. 

