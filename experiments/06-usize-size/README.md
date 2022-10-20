# `usize-size`

This experiment is a simple test to see if the size of a `usize` in wasm is the same as the size of a `usize` on the 
target host.

## How are we computing usize's size?

We are converting a number of type usize to bytes and finding the number of bytes in the resulting byte array: 
`1usize.to_ne_bytes().len() as i32`.


## Results

Rust doc says [the size of usize is 8 bytes](https://doc.rust-lang.org/std/primitive.usize.html#method.to_ne_bytes),
unconditionally, whereas this depends on the platform as you see:

```shell
cargo run
   Compiling host v0.1.0 (/Users/amitu/Projects/fpm/experiments/06-usize-size/host)
    Finished dev [unoptimized + debuginfo] target(s) in 2.63s
     Running `target/debug/host`
wasm said: 4
host usize: 8
```

This matches with [portability documentation](https://webassembly.org/docs/portability/) for wasm as well:

> Memory regions which can be efficiently addressed with 32-bit pointers or indices.

There exists [a proposal to add a 64-bit memory addressing mode to 
wasm](https://github.com/WebAssembly/memory64/blob/main/proposals/memory64/Overview.md), but it is not yet implemented.

So unless otherwise stated, size of usize in wasm is 4, and not 8 as Rust doc says.

You can also this in an error message I got when I was trying something:

```shell
amitu@192 guest % cargo check
    Checking guest v0.1.0 (/Users/amitu/Projects/fpm/experiments/06-usize-size/guest)
error[E0369]: cannot add `[u8; 8]` to `[u8; 8]`
  --> src/lib.rs:13:40
   |
13 |         let l = self.len.to_ne_bytes() + (self.data as usize).to_ne_bytes();
   |                 ---------------------- ^ ---------------------------------- [u8; 8]
   |                 |
   |                 [u8; 8]

For more information about this error, try `rustc --explain E0369`.
error: could not compile `guest` due to previous error
amitu@192 guest % cargo build --target wasm32-unknown-unknown
   Compiling guest v0.1.0 (/Users/amitu/Projects/fpm/experiments/06-usize-size/guest)
error[E0369]: cannot add `[u8; 4]` to `[u8; 4]`
  --> src/lib.rs:13:40
   |
13 |         let l = self.len.to_ne_bytes() + (self.data as usize).to_ne_bytes();
   |                 ---------------------- ^ ---------------------------------- [u8; 4]
   |                 |
   |                 [u8; 4]

For more information about this error, try `rustc --explain E0369`.
error: could not compile `guest` due to previous error
```

