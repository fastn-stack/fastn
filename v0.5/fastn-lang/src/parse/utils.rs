impl fastn_section::parse::Document {
    pub fn new(
        document: fastn_section::token::Document,
    ) -> (fastn_section::parse::Document, Vec<fastn_section::Section>) {
        (
            fastn_section::parse::Document {
                module_doc: document.module_doc,
                imports: vec![],
                definitions: Default::default(),
                content: vec![],
                errors: document.errors,
                warnings: document.warnings,
                comments: document.comments,
                line_starts: document.line_starts,
            },
            document.sections,
        )
    }
}
