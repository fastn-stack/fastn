#![allow(dead_code)]

mod angle_text;
mod identifier;
mod kind;
mod module_name;
mod package_name;
mod qualified_identifier;
mod scanner;
mod visibility;

pub use identifier::identifier;
pub use module_name::module_name;
pub use package_name::package_name;
use scanner::Scanner;

impl fastn_p1::ParseOutput {
    pub fn parse_v4(source: &str) -> fastn_p1::ParseOutput {
        let _scanner = scanner::Scanner::new(source);
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
    let mut scanner = fastn_p1::parser::Scanner::new(source);
    let result = f(&mut scanner);
    assert_eq!(result.debug(source), debug);
    assert_eq!(scanner.remaining(), remaining);
}