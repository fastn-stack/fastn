extern crate self as fpm;

pub mod build;
pub mod library;

pub use build::build;
pub use library::Library;
