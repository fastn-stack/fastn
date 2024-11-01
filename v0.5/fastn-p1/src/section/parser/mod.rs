pub(super) mod identifier;
pub(super) mod kind;
pub(super) mod kinded_name;
pub(super) mod module_name;
pub(super) mod package_name;
pub(super) mod qualified_identifier;
pub(super) mod scanner;
pub(super) mod section_init;
pub(super) mod ses;
pub(super) mod visibility;

pub use scanner::Scanner;

impl fastn_p1::unresolved::Document {
    pub fn parse(source: &str) -> fastn_p1::unresolved::Document {
        let _scanner = fastn_p1::section::Scanner::new(source, Default::default());
        todo!()
    }
}

#[cfg(test)]
#[track_caller]
fn p<T: fastn_p1::debug::JDebug, F: FnOnce(&mut fastn_p1::section::Scanner) -> T>(
    source: &str,
    f: F,
    debug: serde_json::Value,
    remaining: &str,
) {
    let mut scanner = fastn_p1::section::Scanner::new(source, Default::default());
    let result = f(&mut scanner);
    assert_eq!(result.debug(source), debug);
    assert_eq!(scanner.remaining(), remaining);
}

#[macro_export]
macro_rules! tt {
    ($f:expr) => {
        #[allow(unused_macros)]
        macro_rules! t {
            ($source:expr, $debug:tt, $remaining:expr) => {
                fastn_p1::section::parser::p($source, $f, serde_json::json!($debug), $remaining);
            };
            ($source:expr, $debug:tt) => {
                fastn_p1::section::parser::p($source, $f, serde_json::json!($debug), "");
            };
        }
        #[allow(unused_macros)]
        macro_rules! f {
            ($source:expr) => {
                fastn_p1::section::parser::p($source, $f, serde_json::json!(null), $source);
            };
        }
    };
}
