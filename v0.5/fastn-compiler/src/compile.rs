struct Compiler {
    symbols: Box<dyn fastn_compiler::SymbolStore>,
    interner: string_interner::DefaultStringInterner,
    #[expect(unused)]
    bag: std::collections::HashMap<fastn_compiler::Symbol, fastn_compiler::LookupResult>,
    #[expect(unused)]
    auto_imports: Vec<fastn_section::AutoImport>,
}

impl Compiler {
    fn new(
        symbols: Box<dyn fastn_compiler::SymbolStore>,
        auto_imports: Vec<fastn_section::AutoImport>,
    ) -> Self {
        Self {
            symbols,
            interner: string_interner::StringInterner::new(),
            bag: std::collections::HashMap::new(),
            auto_imports,
        }
    }

    fn update_partially_resolved(
        &mut self,
        _partially_resolved: Vec<fastn_unresolved::Definition>,
    ) {
        todo!()
    }

    async fn fetch_unresolved_symbols(
        &mut self,
        symbols_to_fetch: &mut [fastn_unresolved::SymbolName],
    ) {
        let _found = self.symbols.lookup(&mut self.interner, symbols_to_fetch);
    }

    /// try to resolve as many symbols as possible, and return the ones that we made any progress on.
    ///
    /// this function should be called in a loop, until the list of symbols is empty.
    fn resolve_symbols(
        &mut self,
        _doc: &mut fastn_unresolved::Document,
        _symbols: &mut [fastn_unresolved::SymbolName],
    ) -> Vec<fastn_unresolved::Definition> {
        todo!()
    }

    /// try to make as much progress as possibly by resolving as many symbols as possible, and return
    /// the vec of ones that could not be resolved.
    ///
    /// it also returns vec of partially resolved symbols, so we do not directly modify the bag, we want
    /// all bag updates to happen in one place.
    ///
    /// if this returns an empty list of symbols, we can go ahead and generate the JS.
    fn resolve_document(
        &mut self,
        d: &mut fastn_unresolved::Document,
    ) -> (
        Vec<fastn_unresolved::SymbolName>,
        Vec<fastn_unresolved::Definition>,
    ) {
        for ci in &d.content {
            if let fastn_unresolved::UR::UnResolved(_c) = ci {
                todo!()
            }
        }

        todo!()
    }

    async fn compile(
        &mut self,
        mut d: fastn_unresolved::Document,
    ) -> Result<fastn_compiler::Output, fastn_compiler::Error> {
        // we only make 10 attempts to resolve the document: we need a warning if we are not able to
        // resolve the document in 10 attempts.
        for _ in 1..10 {
            // resolve_document can internally run in parallel.
            let (mut unresolved_symbols, partially_resolved) = self.resolve_document(&mut d);
            self.update_partially_resolved(partially_resolved);
            if unresolved_symbols.is_empty() {
                break;
            }

            self.fetch_unresolved_symbols(&mut unresolved_symbols).await;
            // this itself has to happen in a loop. we need a warning if we are not able to resolve all
            // symbols in 10 attempts.
            for _ in 1..10 {
                // resolve_document can internally run in parallel.
                let partially_resolved = self.resolve_symbols(&mut d, &mut unresolved_symbols);
                self.update_partially_resolved(partially_resolved);

                if unresolved_symbols.is_empty() {
                    break;
                }
                self.fetch_unresolved_symbols(&mut unresolved_symbols).await;
            }

            if !unresolved_symbols.is_empty() {
                // we were not able to resolve all symbols
            }
        }

        todo!()
    }
}

/// this is our main compiler module
///
/// it should be called with a parsed document, and it returns generated JS.
///
/// on success, we return the JS, and list of warnings, and on error, we return the list of
/// diagnostics, which is an enum containing warning and error.
///
/// earlier we had strict mode here, but to simplify things, now we let the caller convert non-empty
/// warnings from OK part as error, and discard the generated JS.
pub async fn compile(
    symbols: Box<dyn fastn_compiler::SymbolStore>,
    document_id: &fastn_unresolved::ModuleName,
    source: &str,
    auto_imports: Vec<fastn_section::AutoImport>,
) -> Result<fastn_compiler::Output, fastn_compiler::Error> {
    Compiler::new(symbols, auto_imports)
        .compile(fastn_unresolved::parse(document_id, source))
        .await
}
