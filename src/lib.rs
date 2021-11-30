extern crate self as fpm;

pub mod build;
pub mod check;
pub mod config;
pub mod library;

pub use build::build;
pub use check::check;
pub use config::Dependency;
pub use config::Package;
pub use library::Library;

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
