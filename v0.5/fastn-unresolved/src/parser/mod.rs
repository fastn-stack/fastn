mod component_invocation;
mod function_definition;
mod import;

pub fn parse(
    main_package: &fastn_package::MainPackage,
    module: fastn_section::Module,
    source: &str,
    arena: &mut fastn_section::Arena,
) -> fastn_unresolved::Document {
    let package_name = module.package(arena).to_string();
    let (mut document, sections) = fastn_unresolved::Document::new(
        module,
        fastn_section::Document::parse(&arcstr::ArcStr::from(source), module),
        arena,
    );

    let package = main_package.packages.get(&package_name);
    // todo: first go through just the imports and desugar them

    // guess the section and call the appropriate unresolved method.
    for section in sections.into_iter() {
        let name = section.simple_name().map(|v| v.to_ascii_lowercase());
        let kind = section
            .simple_section_kind_name()
            .map(str::to_ascii_lowercase);
        // at this level we are very liberal, we just need a hint to which parser to use.
        // the parsers themselves do the error checks and validation.
        //
        // at this point, we have to handle correct and incorrect documents both.
        // people can do all sorts of errors, e.g. `-- import(): foo`, should this go to import of
        // function parser?
        // people can make typos, e.g., `compnent` instead of `component`.
        // so if we do not get exact match, we try to do recovery (maybe we can use
        // https://github.com/lotabout/fuzzy-matcher).
        match (
            kind.as_deref(),
            name.as_deref(),
            section.init.function_marker.is_some(),
        ) {
            (Some("import"), _, _) | (_, Some("import"), _) => import::import(
                section,
                &mut document,
                arena,
                &package,
                main_package.name.as_str(),
            ),
            (Some("record"), _, _) => todo!(),
            (Some("type"), _, _) => todo!(),
            (Some("module"), _, _) => todo!(),
            (Some("component"), _, _) => todo!(),
            (_, _, true) => {
                function_definition::function_definition(section, &mut document, arena, &package)
            }
            (None, _, _) => {
                component_invocation::component_invocation(section, &mut document, arena, &package)
            }
            (_, _, _) => todo!(),
        }
    }

    // document.add_definitions_to_scope(arena, global_aliases);
    document
}

#[cfg(test)]
#[track_caller]
/// t1 takes a function parses a single section. and another function to test the parsed document
fn t1<PARSER, TESTER>(source: &str, expected: serde_json::Value, parser: PARSER, tester: TESTER)
where
    PARSER: Fn(
        fastn_section::Section,
        &mut fastn_unresolved::Document,
        &mut fastn_section::Arena,
        &Option<&fastn_package::Package>,
    ),
    TESTER: FnOnce(fastn_unresolved::Document, serde_json::Value, &fastn_section::Arena),
{
    println!("--------- testing -----------\n{source}\n--------- source ------------");

    let mut arena = fastn_section::Arena::default();
    let module = fastn_section::Module::main(&mut arena);
    let (mut document, sections) = fastn_unresolved::Document::new(
        module,
        fastn_section::Document::parse(&arcstr::ArcStr::from(source), module),
        &mut arena,
    );

    let section = {
        assert_eq!(sections.len(), 1);
        sections.into_iter().next().unwrap()
    };

    // assert everything else is empty
    parser(section, &mut document, &mut arena, &None);

    // Ensure t!() fails if there are any errors - use t_err!() for cases with errors
    assert!(
        document.errors.is_empty(),
        "t!() should not be used when errors are expected. Use t_err!() instead. Errors: {:?}",
        document.errors
    );

    tester(document, expected, &arena);
}

#[cfg(test)]
#[track_caller]
pub fn f1<PARSER>(source: &str, expected_errors: serde_json::Value, parser: PARSER)
where
    PARSER: Fn(
        fastn_section::Section,
        &mut fastn_unresolved::Document,
        &mut fastn_section::Arena,
        &Option<&fastn_package::Package>,
    ),
{
    println!("--------- testing failure -----------\n{source}\n--------- source ------------");

    let mut arena = fastn_section::Arena::default();
    let module = fastn_section::Module::main(&mut arena);
    let parsed_doc = fastn_section::Document::parse(&arcstr::ArcStr::from(source), module);
    let (mut document, sections) = fastn_unresolved::Document::new(module, parsed_doc, &mut arena);

    if sections.len() == 1 {
        let section = sections.into_iter().next().unwrap();
        parser(section, &mut document, &mut arena, &None);
    }

    // Check that we have the expected errors
    let actual_errors: Vec<String> = document
        .errors
        .iter()
        .map(|e| format!("{:?}", e.value))
        .collect();

    let expected_errors = if expected_errors.is_array() {
        expected_errors
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e.as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    } else if expected_errors.is_string() {
        vec![expected_errors.as_str().unwrap().to_string()]
    } else {
        panic!("Expected errors must be a string or array of strings");
    };

    assert_eq!(
        actual_errors, expected_errors,
        "Error mismatch for source: {source}"
    );
}

#[cfg(test)]
#[track_caller]
fn t_err1<PARSER, TESTER>(
    source: &str,
    expected: serde_json::Value,
    expected_errors: serde_json::Value,
    parser: PARSER,
    tester: TESTER,
) where
    PARSER: Fn(
        fastn_section::Section,
        &mut fastn_unresolved::Document,
        &mut fastn_section::Arena,
        &Option<&fastn_package::Package>,
    ),
    TESTER: FnOnce(fastn_unresolved::Document, serde_json::Value, &fastn_section::Arena),
{
    println!("--------- testing with errors -----------\n{source}\n--------- source ------------");

    let mut arena = fastn_section::Arena::default();
    let module = fastn_section::Module::main(&mut arena);
    let (mut document, sections) = fastn_unresolved::Document::new(
        module,
        fastn_section::Document::parse(&arcstr::ArcStr::from(source), module),
        &mut arena,
    );

    let section = {
        assert_eq!(sections.len(), 1);
        sections.into_iter().next().unwrap()
    };

    parser(section, &mut document, &mut arena, &None);

    // Check errors
    let actual_errors: Vec<String> = document
        .errors
        .iter()
        .map(|e| format!("{:?}", e.value))
        .collect();

    let expected_errors_vec = if expected_errors.is_array() {
        expected_errors
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e.as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    } else if expected_errors.is_string() {
        vec![expected_errors.as_str().unwrap().to_string()]
    } else {
        panic!("Expected errors must be a string or array of strings");
    };

    assert_eq!(
        actual_errors, expected_errors_vec,
        "Error mismatch for source: {source}"
    );

    // Clear errors before calling tester (since tester doesn't expect errors)
    document.errors.clear();

    tester(document, expected, &arena);
}

#[cfg(test)]
#[macro_export]
macro_rules! tt_error {
    ($p:expr) => {
        #[allow(unused_macros)]
        macro_rules! f {
            ($source:expr, $expected_errors:tt) => {
                fastn_unresolved::parser::f1(
                    indoc::indoc!($source),
                    serde_json::json!($expected_errors),
                    $p,
                );
            };
        }

        #[allow(unused_macros)]
        macro_rules! f_raw {
            ($source:expr, $expected_errors:tt) => {
                fastn_unresolved::parser::f1($source, serde_json::json!($expected_errors), $p);
            };
        }
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! tt {
    ($p:expr, $d:expr) => {
        #[allow(unused_macros)]
        macro_rules! t {
            ($source:expr, $expected:tt) => {
                fastn_unresolved::parser::t1(
                    indoc::indoc!($source),
                    serde_json::json!($expected),
                    $p,
                    $d,
                );
            };
        }

        #[allow(unused_macros)]
        macro_rules! t_raw {
            ($source:expr, $expected:tt) => {
                fastn_unresolved::parser::t1($source, serde_json::json!($expected), $p, $d);
            };
        }

        #[allow(unused_macros)]
        macro_rules! f {
            ($source:expr, $expected_errors:tt) => {
                fastn_unresolved::parser::f1(
                    indoc::indoc!($source),
                    serde_json::json!($expected_errors),
                    $p,
                );
            };
        }

        #[allow(unused_macros)]
        macro_rules! f_raw {
            ($source:expr, $expected_errors:tt) => {
                fastn_unresolved::parser::f1($source, serde_json::json!($expected_errors), $p);
            };
        }

        #[allow(unused_macros)]
        macro_rules! t_err {
            ($source:expr, $expected:tt, $expected_errors:tt) => {
                fastn_unresolved::parser::t_err1(
                    indoc::indoc!($source),
                    serde_json::json!($expected),
                    serde_json::json!($expected_errors),
                    $p,
                    $d,
                );
            };
        }

        #[allow(unused_macros)]
        macro_rules! t_err_raw {
            ($source:expr, $expected:tt, $expected_errors:tt) => {
                fastn_unresolved::parser::t_err1(
                    $source,
                    serde_json::json!($expected),
                    serde_json::json!($expected_errors),
                    $p,
                    $d,
                );
            };
        }
    };
}
