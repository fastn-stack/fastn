extern crate self as fpm;

pub mod build;
pub mod check;
pub mod config;
pub mod library;
pub mod setup;

pub use build::build;
pub use check::check;
pub use config::Config;
pub use library::Library;
pub use setup::setup;

pub fn fpm_ftd() -> &'static str {
    include_str!("../fpm.ftd")
}

#[cfg(test)]
mod tests {

    #[test]
    fn fbt() {
        if fbt_lib::main().is_some() {
            panic!("test failed")
        }
    }
}
