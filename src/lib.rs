extern crate self as fpm;

pub mod build;
pub mod check;
pub mod config;
pub mod library;

pub use build::build;
pub use check::check;
pub use config::Config;
pub use library::Library;

#[cfg(test)]
mod tests {

    #[test]
    fn fbt() {
        if fbt_lib::main().is_some() {
            panic!("test failed")
        }
    }
}
