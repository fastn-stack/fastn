# Browser

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

## Main On Server Vs Main In Browser

We have two main way to construct the initial DOM tree. 

First is we can construct the DOM tree on server side, by running the `doc.wasm`'s `main()` on server side, constructing 
a `internal-dom` (refer [`dom.md`](dom.md) for details) (along with memory snapshot, `doc.json`), and serializing the 
`internal-dom` to HTML. Finally in browser we attach the event handlers, and proxy dom mutation methods exposed by 
`runtime.wasm` to real browser DOM. Note that since `main()` has already run on server, we do not run it in browser.
Let's called it SSR or hydration method.

The second possibility is we do not run `doc.wasm` on server at all, do not send `doc.json`, let `doc.html` have an
empty body, and `linker.js` injected, and let `linker.js` run the `main()` method of `doc.wasm`. Since `main()` is
responsible for memory initialisation and initial DOM construction, this work.

For clarity lets call `doc.ssr.html` which contains serialised DOM, the output of `main on server` method, and 
`doc.html` for when we run main in browser.

### Note For Static Building

We can still use the main in server approach in static build mode (we run `fastn build`, which generates a `.build` 
folder containing all static files, which is deployed on a static hosting provider). We will have to store the generated
`doc.json` file as a separate artifact, or we can inline the `doc.json` in `doc.ssr.html` itself.

### Consideration: Google Bots etc

In case of google bot etc, the `linker.js` logic, we should return `doc.ssr.minus-linker.html`, as Google etc do not 
trigger event handlers. 

It is possible that Google does event triggers in the craw process, for example if your page has multiple tabs, and 
only one tab is open, and the individual tabs do not have dedicated URLs, then Google bot will never discover the 
content of the other tabs unless google bot "clicks" on the other tabs.

This is a tradeoff for the entire wasm based approach itself, it will only work if google bot runs wasm as well, which
we do not know.

### Consideration: Page Size

`doc.ssr.html` is going to be bigger than `doc.html` as later does not contain server rendered HTML. We have discussed
compilation approach which generates two wasm files, `doc.wasm` and `doc.without-main.wasm` files. If the `HTML_Delta >
WASM_Delta` then for browsers (not crawlers) the optimal approach could be to send `doc.html` and `doc.wasm` instead of
`doc.ssr.html` + `doc.without-main.wasm`.

### Decision

ssr only for bots and `fastn static` build. Because in static we do not have any way to serve different content based on 
user agent, if we could even in `fastn static` we will not send `doc.ssr.html` to regular browsers.

