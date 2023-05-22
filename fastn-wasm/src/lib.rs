extern crate self as fastn_wasm;

mod ast;
mod encoder;

pub use ast::*;
pub use encoder::encode;