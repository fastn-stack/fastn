# Strings

[Back To Design Home](./).

We store strings like everything else in `fastn_runtime::Memory.string`.

## String Constants

A FTD file may contain some string constants. Like:

```ftd
-- ftd.text: hello world
```

### An Alternative: Store String Constants In WASM

Here the string `hello world` is never modified, and can be made part of the program, the wasm file. Many compilers do
just that, and if the wasm file is all we had access to, we would also do the same.

We can use `(data)` segment, and store each string in the wasm file.

```wat
(module
    (memory 1)                         ;; enough memory to hold all strings
    (data (i32.const 0) 9)             ;; "hello world".len() == 9
    (data (i32.const 1) "hello world") ;; offset is 1 now
    (data (i32.const 10) 2)            ;; storing "hi" next
    (data (i32.const 11) "hi") 
)
```

Note that in our design we do not store any of the ftd variables in wasm memory, they all are kept in 
`fastn_runtime::Memory` so the wasm memory is only used for storing such string constants.

We can refer to string by their start address (which would be the location where we store the length, so we have to
keep track of only one number to identify each string).

When wasm instance is created we can scan the entire memory, and extract each string out.

### Problem With Storing Constants In WASM

In the browser (check `browser.md` for details), we download `doc.wasm`, which can contain the strings. But we also
download `doc.json`, the serialised snapshot of `fastn_runtime::Memory`, which will also contain the strings. So if
we store strings in `doc.wasm` we end up downloading them twice.

So our compilation process will create both `doc.wasm` and `doc-constants.json`. `doc-constants.json` would be read by
the server (check out `server.md` file for detail), and content would be serialised into the in memory version of
`fastn_runtime::Memory` before the `doc.wasm`'s `main()` is called. 