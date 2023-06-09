# `linker.js`

Read [`browser.md`](browser.md) first for context.

Once `doc.html` loads, it loads `linker.js`, which downloads `doc.json`, `doc.wasm`, `runtime.wasm` to make the
event handlers work.

`linker.js` creates two wasm instances, one for `runtime.wasm` and the other for `doc.wasm`. The `runtime instance` is 
fed `doc.json` as we are not going to call the `main` function of `doc.wasm` from browser, as main's job was to create 
the  DOM tree, and initialise `fastn_runtime::Memory`.

`doc.wasm` is created with assumption that a bunch of functions are exported by the host (eg `create_i32()`,
`create_kernel()` and so on). And `doc.wasm` itself exports `main()`, `call_by_index(idx: i32, func_data:
fastn_runtime::Pointer) -> fastn_runtime::Pointer` and `void_by_index(idx: i32, func_data: fastn_runtime::Pointer)`.

The `doc.call_by_index()` and `doc.void_by_index()` are called from runtime, and they internally call
`runtime.create_kernel()`, `runtime.create_i32()` etc (it can happen recursively as well, eg `runtime.set_i32()` may
trigger `doc.call_by_index()` which may call `runtime.create_kernel()` and on ond on).

`linker.js` connects the two sides. We can do this in two ways, a. wrapper functions, and b. function tables.

## Approach a: Wrapper Functions

`linker.js` can create wrapper functions for the two sides, eg:

```js
const importObject = {
  imports: { 
    // doc_instance will call create_kernel etc, which gets forwarded to runtime_instance
    create_kernel: (arguments) => runtime_instance.exports.create_kernel.apply(null, arguments), 
    .. all the exports from runtime, source: fastn_runtime::Dom::register_functions() ..
    
    // runtime_instance will call call_by_index etc, which gets forwarded to doc_instance
    call_by_index: (arguments) => doc_instance.exports.call_by_index.apply(null, arguments),
    void_by_index: (arguments) => doc_instance.exports.void_by_index.apply(null, arguments),
  },
};

let runtime_instance = null;
let doc_instance = null;

WebAssembly.instantiateStreaming(fetch("runtime.wasm"), importObject).then(
  (obj) => runtime_instance = obj.instance; // check if both are loaded, if so call .start on both
);

WebAssembly.instantiateStreaming(fetch("doc.wasm"), importObject).then(
  (obj) => doc_instance = obj.instance;
);
```

For each method in both the instances, we create a wrapper JS function, and the wrapper will call the corresponding
exported function on the other instance.

Wasm files then import the methods they are interested in, eg `doc.wasm`:

```wat
(module
    (import "fastn" "create_frame" (func $create_frame))
    ..
    
    (func main
        (call create_kernel)
    )
)
```

## Approach b: Function Tables

Wasm has a concept of function tables.

```js
let number_of_exports_in_runtime = 10; // say
let number_of_exports_in_doc = 2; // only call_by_index and void_by_index

var table = new WebAssembly.Table({
    initial: number_of_exports_in_runtime + number_of_exports_in_doc, 
    element: "externref"
});

const importObject = {
    linker: {
        table: table,
    }
}

let runtime_instance = null;
let doc_instance = null;

WebAssembly.instantiateStreaming(fetch("runtime.wasm"), importObject).then(
  (obj) => runtime_instance = obj.instance;
);

WebAssembly.instantiateStreaming(fetch("doc.wasm"), importObject).then(
  (obj) => doc_instance = obj.instance;
);
```

And in our `doc.wasm` file we have:

```wat
(module
    (import "linker" "table" ??)
    (elem (i32.const 0) $call_by_index $void_by_index)
    
    (func $create_kernel
        (call_inderct $create_kernel_type (i32.const 2))
    )
    
    (func $main (export "main")  
        (call $create_kernel)    
    )
    
    (func $call_by_index ..)
    (func $void_by_index ..)  
)
```

`doc.wasm` gets the first two slots, starting from index `0`, in the table to export the two methods. `runtime.wasm`
uses the rest of the slots:

```wat
(module
    (import "linker" "table" ??)
    (elem (i32.const 2) $create_kernel ..)
    
    (func $create_kernel (export "")  ..)
)
```

`runtime.wasm` populates slots starting from index `2`.

When they need to call each other, they use `(call_indirect)` instead of `(call)`.


### Challenge With Table Approach

How to use tables from wasm generated from Rust? Not yet clear.