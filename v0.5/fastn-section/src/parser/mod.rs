mod body;
mod doc_comment;
mod header_value;
mod headers;
mod identifier;
mod identifier_reference;
mod kind;
mod kinded_name;
mod kinded_reference;
mod section;
mod section_init;
mod tes;
mod visibility;

pub use body::body;
pub use doc_comment::doc_comment;
pub use header_value::header_value;
pub use headers::headers;
pub use identifier::identifier;
pub use identifier_reference::identifier_reference;
pub use kind::kind;
pub use kinded_name::kinded_name;
pub use kinded_reference::kinded_reference;
pub use section::section;
pub use section_init::section_init;
pub use tes::{tes_till, tes_till_newline};
pub use visibility::visibility;

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
    loop {
        // Try to parse a section (which will handle its own doc comments)
        if let Some(section) = fastn_section::parser::section(scanner) {
            scanner.output.sections.push(section);
            scanner.skip_spaces();
            scanner.skip_new_lines();
            scanner.skip_spaces();
        } else {
            // No section found - check if there's an orphaned doc comment
            if let Some(doc_span) = fastn_section::parser::doc_comment(scanner) {
                // Orphaned doc comment - report error
                scanner.add_error(doc_span, fastn_section::Error::UnexpectedDocComment);
                scanner.skip_spaces();
                scanner.skip_new_lines();
                scanner.skip_spaces();
                // Continue to try parsing more content
                continue;
            }
            // No more content to parse
            break;
        }
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

#[cfg(test)]
#[track_caller]
fn p_err<
    T: fastn_section::JDebug,
    F: FnOnce(&mut fastn_section::Scanner<fastn_section::Document>) -> T,
>(
    source: &arcstr::ArcStr,
    f: F,
    debug: serde_json::Value,
    remaining: &str,
    expected_errors: serde_json::Value,
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
    assert_eq!(result.debug(), debug, "parsed output mismatch");
    assert_eq!(scanner.remaining(), remaining, "remaining input mismatch");

    // Check errors - extract just the error names
    let errors_debug: Vec<_> = scanner
        .output
        .errors
        .iter()
        .map(|e| {
            // Extract just the error string from {"error": "error_name"}
            use fastn_section::JDebug;
            if let serde_json::Value::Object(map) = e.value.debug() {
                if let Some(serde_json::Value::String(s)) = map.get("error") {
                    s.clone()
                } else {
                    format!("{:?}", e.value)
                }
            } else {
                format!("{:?}", e.value)
            }
        })
        .collect::<Vec<String>>();

    // Convert expected_errors to comparable format
    let expected = match expected_errors {
        serde_json::Value::String(s) => vec![s.as_str().to_string()],
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => vec![],
    };

    assert_eq!(errors_debug, expected, "errors mismatch");
}

#[macro_export]
macro_rules! tt {
    ($f:expr) => {
        #[allow(unused_macros)]
        macro_rules! t {
            ($source:expr, $debug:tt, $remaining:expr) => {
                fastn_section::parser::p(
                    &arcstr::ArcStr::from(indoc::indoc!($source)),
                    $f,
                    serde_json::json!($debug),
                    $remaining,
                );
            };
            ($source:expr, $debug:tt) => {
                fastn_section::parser::p(
                    &arcstr::ArcStr::from(indoc::indoc!($source)),
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
                    &arcstr::ArcStr::from(indoc::indoc!($source)),
                    $f,
                    serde_json::json!(null),
                    $source,
                );
            };
        }
        #[allow(unused_macros)]
        macro_rules! t_err {
            ($source:expr, $debug:tt, $errors:tt, $remaining:expr) => {
                fastn_section::parser::p_err(
                    &arcstr::ArcStr::from(indoc::indoc!($source)),
                    $f,
                    serde_json::json!($debug),
                    $remaining,
                    serde_json::json!($errors),
                );
            };
            ($source:expr, $debug:tt, $errors:tt) => {
                fastn_section::parser::p_err(
                    &arcstr::ArcStr::from(indoc::indoc!($source)),
                    $f,
                    serde_json::json!($debug),
                    "",
                    serde_json::json!($errors),
                );
            };
        }
        // Raw variants that don't use indoc
        #[allow(unused_macros)]
        macro_rules! t_raw {
            ($source:expr, $debug:tt, $remaining:expr) => {
                fastn_section::parser::p(
                    &arcstr::ArcStr::from($source),
                    $f,
                    serde_json::json!($debug),
                    $remaining,
                );
            };
            ($source:expr, $debug:tt) => {
                fastn_section::parser::p(
                    &arcstr::ArcStr::from($source),
                    $f,
                    serde_json::json!($debug),
                    "",
                );
            };
        }
        #[allow(unused_macros)]
        macro_rules! f_raw {
            ($source:expr) => {
                fastn_section::parser::p(
                    &arcstr::ArcStr::from($source),
                    $f,
                    serde_json::json!(null),
                    $source,
                );
            };
        }
        #[allow(unused_macros)]
        macro_rules! t_err_raw {
            ($source:expr, $debug:tt, $errors:tt, $remaining:expr) => {
                fastn_section::parser::p_err(
                    &arcstr::ArcStr::from($source),
                    $f,
                    serde_json::json!($debug),
                    $remaining,
                    serde_json::json!($errors),
                );
            };
            ($source:expr, $debug:tt, $errors:tt) => {
                fastn_section::parser::p_err(
                    &arcstr::ArcStr::from($source),
                    $f,
                    serde_json::json!($debug),
                    "",
                    serde_json::json!($errors),
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

        // Section with doc comment
        t!(
            "
            ;;; Documentation
            -- foo: Hello",
            {
                "sections": [{
                    "init": {
                        "name": "foo",
                        "doc": ";;; Documentation\n"
                    },
                    "caption": ["Hello"]
                }]
            }
        );

        // Multiple sections with doc comments
        t!(
            "
            ;;; First section docs
            -- foo: First
            
            ;;; Second section docs
            -- bar: Second",
            {
                "sections": [
                    {
                        "init": {
                            "name": "foo",
                            "doc": ";;; First section docs\n"
                        },
                        "caption": ["First"]
                    },
                    {
                        "init": {
                            "name": "bar",
                            "doc": ";;; Second section docs\n"
                        },
                        "caption": ["Second"]
                    }
                ]
            }
        );

        // Orphaned doc comment at beginning
        t!(
            "
            ;;; This is orphaned
            
            -- foo: Section",
            {
                "errors": [{
                    "error": "unexpected_doc_comment"
                }],
                "sections": [{
                    "init": {"name": "foo"},
                    "caption": ["Section"]
                }]
            }
        );

        // Multiple orphaned doc comments
        t!(
            "
            ;;; First orphan
            
            ;;; Second orphan
            
            -- foo: Section",
            {
                "errors": [
                    {"error": "unexpected_doc_comment"},
                    {"error": "unexpected_doc_comment"}
                ],
                "sections": [{
                    "init": {"name": "foo"},
                    "caption": ["Section"]
                }]
            }
        );

        // Orphaned doc comment at end of file
        t!(
            "
            -- foo: Section
            
            ;;; This doc comment has no section after it",
            {
                "sections": [{
                    "init": {"name": "foo"},
                    "caption": ["Section"]
                }],
                "errors": [{
                    "error": "unexpected_doc_comment"
                }]
            }
        );

        // Orphaned doc comment between sections (with blank line)
        t!(
            "
            -- foo: First
            
            ;;; Orphaned comment
            
            -- bar: Second",
            {
                "sections": [
                    {
                        "init": {"name": "foo"},
                        "caption": ["First"]
                    },
                    {
                        "init": {"name": "bar"},
                        "caption": ["Second"]
                    }
                ],
                "errors": [{
                    "error": "unexpected_doc_comment"
                }]
            }
        );
    }
}
