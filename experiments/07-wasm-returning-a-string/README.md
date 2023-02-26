# `wasm-returning-a-string`

While working on this experiment, [radu-matei's post on wasm memory](https://radu-matei.com/blog/practical-guide-to-wasm-memory/)
and [his code](https://github.com/radu-matei/wasm-memory/blob/main/src/main.rs) were quite helpful.

## Why do we care about `usize`'s size?

In the [last experiment](../06-usize-size/) we saw that the size of a `usize` in wasm is 4 bytes, not 8 bytes as Rust 
doc says.

As per [Rust docs](https://rust-lang.github.io/unsafe-code-guidelines/layout/scalars.html#isize-and-usize):

> The isize and usize types are pointer-sized signed and unsigned integers. They have the same layout as the pointer
> types for which the pointee is Sized, and are layout compatible with C's uintptr_t and intptr_t types.

So usize and pointers are the same size. Since we are going to return the pointer, and do manual memory management, we
need to be sure about the size of the pointer.

