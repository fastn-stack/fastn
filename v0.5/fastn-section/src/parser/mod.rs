mod body;
mod header_value;
mod headers;
mod identifier;
mod kind;
mod kinded_name;
// mod module_name;
// mod package_name;
// mod qualified_identifier;
mod identifier_reference;
mod section;
mod section_init;
mod visibility;

pub use body::body;
pub use header_value::header_value;
pub use headers::headers;
#[expect(unused)]
pub use identifier::identifier;
pub use identifier_reference::identifier_reference;
pub use kind::kind;
pub use kinded_name::kinded_name;
// pub use module_name::module_name;
// pub use package_name::package_name;
// pub use qualified_identifier::qualified_identifier;
pub use section::section;
pub use section_init::section_init;

impl fastn_section::Document {
    pub fn parse(
        source: &arcstr::ArcStr,
        module: fastn_section::Module,
    ) -> fastn_section::Document {
        let mut scanner = fastn_section::Scanner::new(
            source,
            Default::default(),
            module,
            fastn_section::Document {
                module,
                module_doc: None,
                sections: vec![],
                errors: vec![],
                warnings: vec![],
                comments: vec![],
                line_starts: vec![],
            },
        );
        document(&mut scanner);
        scanner.output
    }
}

pub fn document(scanner: &mut fastn_section::Scanner<fastn_section::Document>) {
    // TODO: parse module_doc, comments etc
    scanner.skip_spaces();
    while let Some(section) = fastn_section::parser::section(scanner) {
        scanner.skip_spaces();
        scanner.skip_new_lines();
        scanner.skip_spaces();
        scanner.output.sections.push(section);
    }
}

#[cfg(test)]
#[track_caller]
fn p<
    T: fastn_section::JDebug,
    F: FnOnce(&mut fastn_section::Scanner<fastn_section::Document>) -> T,
>(
    source: &arcstr::ArcStr,
    f: F,
    debug: serde_json::Value,
    remaining: &str,
) {
    let mut arena = fastn_section::Arena::default();
    let module = fastn_section::Module::main(&mut arena);

    let mut scanner = fastn_section::Scanner::new(
        source,
        Default::default(),
        module,
        fastn_section::Document {
            module,
            module_doc: None,
            sections: vec![],
            errors: vec![],
            warnings: vec![],
            comments: vec![],
            line_starts: vec![],
        },
    );
    let result = f(&mut scanner);
    assert_eq!(result.debug(), debug);
    assert_eq!(scanner.remaining(), remaining);
}

#[macro_export]
macro_rules! tt {
    ($f:expr) => {
        #[allow(unused_macros)]
        macro_rules! t {
            ($source:expr, $debug:tt, $remaining:expr) => {
                fastn_section::parser::p(
                    &arcstr::literal!($source),
                    $f,
                    serde_json::json!($debug),
                    $remaining,
                );
            };
            ($source:expr, $debug:tt) => {
                fastn_section::parser::p(
                    &arcstr::literal!($source),
                    $f,
                    serde_json::json!($debug),
                    "",
                );
            };
        }
        #[allow(unused_macros)]
        macro_rules! f {
            ($source:expr) => {
                fastn_section::parser::p(
                    &arcstr::literal!($source),
                    $f,
                    serde_json::json!(null),
                    $source,
                );
            };
        }
    };
}

#[cfg(test)]
mod test {
    fn doc(
        scanner: &mut fastn_section::Scanner<fastn_section::Document>,
    ) -> fastn_section::Document {
        fastn_section::parser::document(scanner);
        scanner.output.clone()
    }

    fastn_section::tt!(doc);
    #[test]
    fn document() {
        t!(
            "-- foo: Hello World",
            {
                "sections": [{
                    "init": {"name": "foo"},
                    "caption": ["Hello World"]
                }]
            }
        );

        t!(
            "-- foo: Hello World from foo\n-- bar: Hello World from bar",
            {
                "sections": [
                    {
                        "init": {"name": "foo"},
                        "caption": ["Hello World from foo"]
                    },
                    {
                        "init": {"name": "bar"},
                        "caption": ["Hello World from bar"]
                    }
                ]
            }
        );
    }
}
