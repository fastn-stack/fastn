pub struct Compiler {
    unresolved: std::collections::HashMap<
        fastn_lang::section::Identifier,
        fastn_lang::unresolved::Definition,
    >,
    resolved: std::collections::HashMap<
        fastn_lang::section::Identifier,
        fastn_lang::resolved::Definition,
    >,
}

enum CompilerState {
    Done(Compiler),
    StuckOnDocuments(Compiler, Vec<fastn_lang::Span>),
}

impl Compiler {
    pub fn compile(_source: &str, _name: &str) -> CompilerState {
        todo!()
    }

    pub fn continue_after_documents(
        self,
        _source: &str,
        _documents: std::collections::HashMap<fastn_lang::Span, &str>,
    ) -> CompilerState {
        todo!()
    }
}
