impl fastn_lang::unresolved::Document {
    pub fn new(
        document: fastn_lang::section::Document,
    ) -> (fastn_lang::unresolved::Document, Vec<fastn_lang::Section>) {
        (
            fastn_lang::unresolved::Document {
                module_doc: document.module_doc,
                imports: vec![],
                definitions: Default::default(),
                content: vec![],
                errors: document.errors,
                comments: document.comments,
                line_starts: document.line_starts,
            },
            document.sections,
        )
    }
}
