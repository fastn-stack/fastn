#[track_caller]
pub fn p<
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

    // Ensure no errors were generated
    assert!(
        scanner.output.errors.is_empty(),
        "Unexpected errors in test: {:?}",
        scanner.output.errors
    );
}

#[track_caller]
pub fn p_err<
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
                fastn_section::parser::test::p(
                    &arcstr::ArcStr::from(indoc::indoc!($source)),
                    $f,
                    serde_json::json!($debug),
                    $remaining,
                );
            };
            ($source:expr, $debug:tt) => {
                fastn_section::parser::test::p(
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
                fastn_section::parser::test::p(
                    &arcstr::ArcStr::from(indoc::indoc!($source)),
                    $f,
                    serde_json::json!(null),
                    $source,
                );
            };
            ($source:expr, $errors:tt) => {
                fastn_section::parser::test::p_err(
                    &arcstr::ArcStr::from(indoc::indoc!($source)),
                    $f,
                    serde_json::json!(null),
                    $source,
                    serde_json::json!($errors),
                );
            };
        }
        #[allow(unused_macros)]
        macro_rules! t_err {
            ($source:expr, $debug:tt, $errors:tt, $remaining:expr) => {
                fastn_section::parser::test::p_err(
                    &arcstr::ArcStr::from(indoc::indoc!($source)),
                    $f,
                    serde_json::json!($debug),
                    $remaining,
                    serde_json::json!($errors),
                );
            };
            ($source:expr, $debug:tt, $errors:tt) => {
                fastn_section::parser::test::p_err(
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
                fastn_section::parser::test::p(
                    &arcstr::ArcStr::from($source),
                    $f,
                    serde_json::json!($debug),
                    $remaining,
                );
            };
            ($source:expr, $debug:tt) => {
                fastn_section::parser::test::p(
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
                fastn_section::parser::test::p(
                    &arcstr::ArcStr::from($source),
                    $f,
                    serde_json::json!(null),
                    $source,
                );
            };
            ($source:expr, $errors:tt) => {
                fastn_section::parser::test::p_err(
                    &arcstr::ArcStr::from($source),
                    $f,
                    serde_json::json!(null),
                    $source,
                    serde_json::json!($errors),
                );
            };
        }
        #[allow(unused_macros)]
        macro_rules! t_err_raw {
            ($source:expr, $debug:tt, $errors:tt, $remaining:expr) => {
                fastn_section::parser::test::p_err(
                    &arcstr::ArcStr::from($source),
                    $f,
                    serde_json::json!($debug),
                    $remaining,
                    serde_json::json!($errors),
                );
            };
            ($source:expr, $debug:tt, $errors:tt) => {
                fastn_section::parser::test::p_err(
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
