extern crate self as fpm;

pub mod build;
pub mod fpm_check;
pub mod fpm_config;
pub mod library;

pub use build::build;
pub use fpm_check::fpm_check;
pub use fpm_config::FPMConfig;
pub use library::Library;
