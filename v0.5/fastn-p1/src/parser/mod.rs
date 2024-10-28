#![allow(dead_code)]

mod identifier;
mod kind;
mod kinded_name;
mod module_name;
mod package_name;
mod qualified_identifier;
mod scanner;
mod section_init;
mod visibility;

pub use identifier::identifier;
pub use kind::kind;
pub use module_name::module_name;
pub use package_name::package_name;
pub use qualified_identifier::qualified_identifier;
use scanner::Scanner;

impl fastn_p1::ParseOutput {
    pub fn parse_v4(source: &str) -> fastn_p1::ParseOutput {
        let _scanner = fastn_p1::parser::Scanner::new(source, Default::default());
        todo!()
    }
}

#[cfg(test)]
#[track_caller]
fn p<T: fastn_p1::debug::JDebug, F: FnOnce(&mut fastn_p1::parser::Scanner) -> T>(
    source: &str,
    f: F,
    debug: serde_json::Value,
    remaining: &str,
) {
    let mut scanner = fastn_p1::parser::Scanner::new(source, Default::default());
    let result = f(&mut scanner);
    assert_eq!(result.debug(source), debug);
    assert_eq!(scanner.remaining(), remaining);
}

#[macro_export]
macro_rules! tt {
    ($f:expr) => {
        macro_rules! t {
            ($source:expr, $debug:tt, $remaining:expr) => {
                fastn_p1::parser::p($source, $f, serde_json::json!($debug), $remaining);
            };
        }
    };
}
