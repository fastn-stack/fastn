#![warn(unused_extern_crates)]
#![deny(unused_crate_dependencies)]

// extern crate self as fastn_ssh;

mod init;
mod run;

pub use init::init;
pub use run::run;
