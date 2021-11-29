extern crate self as fpm;

pub mod build;
pub mod fpm_check;
pub mod library;

pub use build::build;
pub use fpm_check::fpm_check;
pub use library::Library;
