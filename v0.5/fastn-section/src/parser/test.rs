/// Helper function to check parser invariants
#[track_caller]
fn check_invariants<'a>(
    scanner: &fastn_section::Scanner<'a, fastn_section::Document>,
    start_index: fastn_section::scanner::Index<'a>,
    initial_error_count: usize,
    result_debug: &serde_json::Value,
    source: &str,
) {
    let end_index = scanner.index();
    let final_error_count = scanner.output.errors.len();
    let scanner_advanced = start_index != end_index;
    let errors_added = final_error_count > initial_error_count;
    let has_result = *result_debug != serde_json::Value::Null;

    // Invariant 1: If errors added, scanner must advance
    assert!(
        !errors_added || scanner_advanced,
        "Invariant violation: Parser added {} error(s) but didn't advance scanner! Input: {:?}",
        final_error_count - initial_error_count,
        source
    );

    // Invariant 2: Scanner must not advance unless it produces result or error
    assert!(
        !scanner_advanced || has_result || errors_added,
        "Invariant violation: Parser advanced scanner but returned null without adding errors! Input: {source:?}"
    );

    // Invariant 3: If parser returns None without errors, scanner should be reset
    assert!(
        has_result || errors_added || !scanner_advanced,
        "Invariant violation: Parser returned None without errors but didn't reset scanner! Input: {source:?}"
    );

    // Invariant 4: All error spans should be non-empty and within consumed range
    if errors_added {
        for error in &scanner.output.errors[initial_error_count..] {
            let span_start = error.span.start();
            let span_end = error.span.end();

            // Check span is non-empty
            assert!(
                span_start < span_end,
                "Invariant violation: Error has empty span! Error: {:?}, Input: {:?}",
                error.value,
                source
            );

            // Check span is within the range that was consumed
            // The span should start at or after where we started parsing
            let start_pos = start_index.pos();
            assert!(
                span_start >= start_pos,
                "Invariant violation: Error span starts before parser started! Error: {:?}, Span start: {}, Parser start: {}, Input: {:?}",
                error.value,
                span_start,
                start_pos,
                source
            );

            // If scanner advanced, error span should be within consumed range
            if scanner_advanced {
                let end_pos = end_index.pos();
                assert!(
                    span_end <= end_pos,
                    "Invariant violation: Error span extends beyond consumed input! Error: {:?}, Span end: {}, Scanner end: {}, Input: {:?}",
                    error.value,
                    span_end,
                    end_pos,
                    source
                );
            }
        }
    }

    // Invariant 5: If we have both result and errors (error recovery case),
    // we should have consumed at least as much as the error spans cover
    if has_result && errors_added {
        let max_error_end = scanner.output.errors[initial_error_count..]
            .iter()
            .map(|e| e.span.end())
            .max()
            .unwrap_or(0);

        let end_pos = end_index.pos();
        assert!(
            end_pos >= max_error_end,
            "Invariant violation: Parser consumed less than error spans! Scanner end: {end_pos}, Max error end: {max_error_end}, Input: {source:?}"
        );
    }
}

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

    // Track initial state for invariant checking
    let start_index = scanner.index();
    let initial_error_count = scanner.output.errors.len();

    let result = f(&mut scanner);

    // Check invariants
    check_invariants(
        &scanner,
        start_index,
        initial_error_count,
        &debug,
        source.as_str(),
    );

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

    // Track initial state for invariant checking
    let start_index = scanner.index();
    let initial_error_count = scanner.output.errors.len();

    let result = f(&mut scanner);

    // Check invariants
    check_invariants(
        &scanner,
        start_index,
        initial_error_count,
        &debug,
        source.as_str(),
    );

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
