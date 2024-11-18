mod component_invocation;
mod import;

pub fn parse(
    _document_id: &fastn_unresolved::ModuleName,
    source: &str,
    source_symbol: string_interner::DefaultSymbol,
) -> fastn_unresolved::Document {
    let (mut document, sections) =
        fastn_unresolved::Document::new(fastn_section::Document::parse(source, source_symbol));
    // guess the section and call the appropriate unresolved method.
    for section in sections.into_iter() {
        let name = section.name(source).to_ascii_lowercase();
        let kind = section.kind_name(source).map(str::to_ascii_lowercase);
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
            name.as_str(),
            section.function_marker.is_some(),
        ) {
            (Some("import"), _, _) | (_, "import", _) => {
                import::import(source, section, &mut document)
            }
            (Some("record"), _, _) => todo!(),
            (Some("type"), _, _) => todo!(),
            (Some("module"), _, _) => todo!(),
            (Some("component"), _, _) => todo!(),
            (None, _, _) => {
                component_invocation::component_invocation(source, section, &mut document)
            }
            (_, _, _) => todo!(),
        }
    }

    document
}

#[cfg(test)]
#[track_caller]
/// t1 takes a function parses a single section. and another function to extract the debug value
fn t1<PARSER, TESTER>(source: &str, expected: serde_json::Value, parser: PARSER, tester: TESTER)
where
    PARSER: Fn(&str, fastn_section::Section, &mut fastn_unresolved::Document),
    TESTER: FnOnce(fastn_unresolved::Document, &str, serde_json::Value),
{
    use string_interner::Symbol;
    println!("--------- testing -----------\n{source}\n--------- source ------------");

    let (mut document, sections) = fastn_unresolved::Document::new(fastn_section::Document::parse(
        source,
        string_interner::DefaultSymbol::try_from_usize(1).unwrap(),
    ));

    let section = {
        assert_eq!(sections.len(), 1);
        sections.into_iter().next().unwrap()
    };

    // assert everything else is empty
    parser(source, section, &mut document);

    tester(document, source, expected);
}

#[cfg(test)]
#[macro_export]
macro_rules! tt {
    ($p:expr, $d:expr) => {
        #[allow(unused_macros)]
        macro_rules! t {
            ($source:expr, $expected:tt) => {
                fastn_unresolved::parser::t1($source, serde_json::json!($expected), $p, $d);
            };
        }
    };
}
