mod import;

pub fn parse(source: &str) -> fastn_section::parse::Document {
    let (mut document, sections) =
        fastn_section::parse::Document::new(fastn_section::token::Document::parse(source));
    // guess the section and call the appropriate unresolved method.
    for section in sections.into_iter() {
        let name = section.name(source).to_ascii_lowercase();
        let kind = section.kind_name(source).to_ascii_lowercase();
        // at this level we are very liberal, we just need a hint to which parser to use.
        // the parsers themselves do the error checks and validation.
        //
        // at this point we we have to handle correct and incorrect document both. people can do
        // all sorts of errors, e.g. `-- import(): foo`, should this go to import of function parser?
        // people can make typos, e.g., compnent instead of component, so if we do not get exact match
        // we try to do recovery. maybe we can use https://github.com/lotabout/fuzzy-matcher
        match (
            kind.as_str(),
            name.as_str(),
            section.function_marker.is_some(),
        ) {
            ("import", _, _) | (_, "import", _) => import::import(source, section, &mut document),
            ("record", _, _) => todo!(),
            ("type", _, _) => todo!(),
            ("module", _, _) => todo!(),
            ("component", _, _) => todo!(),
            (_, _, true) => todo!(),
            (_, _, _) => todo!(),
        }
    }

    document
}
