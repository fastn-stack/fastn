impl fastn_lang::unresolved::Document {
    pub fn new(
        document: fastn_section::Document,
    ) -> (
        fastn_lang::unresolved::Document,
        Vec<fastn_section::Section>,
    ) {
        (
            fastn_lang::unresolved::Document {
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
