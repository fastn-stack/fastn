impl fastn_lang::parse::Document {
    pub fn new(
        document: fastn_lang::token::Document,
    ) -> (fastn_lang::parse::Document, Vec<fastn_lang::Section>) {
        (
            fastn_lang::parse::Document {
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
