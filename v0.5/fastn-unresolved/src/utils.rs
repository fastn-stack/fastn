impl fastn_unresolved::Document {
    pub fn new(
        document: fastn_section::Document,
    ) -> (fastn_unresolved::Document, Vec<fastn_section::Section>) {
        (
            fastn_unresolved::Document {
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
