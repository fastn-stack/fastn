struct Compiler {
    symbols: Box<dyn fastn_compiler::SymbolStore>,
    interner: string_interner::DefaultStringInterner,
    bag: std::collections::HashMap<string_interner::DefaultSymbol, fastn_compiler::LookupResult>,
    #[expect(unused)]
    auto_imports: Vec<fastn_section::AutoImport>,
    #[expect(unused)]
    errors: Vec<fastn_section::Spanned<fastn_section::Error>>,
    #[expect(unused)]
    warnings: Vec<fastn_section::Spanned<fastn_section::Warning>>,
    #[expect(unused)]
    comments: Vec<fastn_section::Span>,
    content: Vec<
        fastn_unresolved::UR<
            fastn_unresolved::ComponentInvocation,
            fastn_type::ComponentInvocation,
        >,
    >,
}

impl Compiler {
    fn new(
        symbols: Box<dyn fastn_compiler::SymbolStore>,
        auto_imports: Vec<fastn_section::AutoImport>,
        document_id: &fastn_unresolved::ModuleName,
        source: &str,
    ) -> Self {
        let d = fastn_unresolved::parse(document_id, source);

        Self {
            symbols,
            interner: string_interner::StringInterner::new(),
            bag: std::collections::HashMap::new(),
            auto_imports,
            errors: d.errors,
            warnings: d.warnings,
            comments: d.comments,
            content: d.content,
        }
    }

    fn update_partially_resolved(&mut self, partially_resolved: Vec<fastn_unresolved::Definition>) {
        for definition in partially_resolved {
            let sym = definition.symbol.unwrap();
            match definition.resolved() {
                Ok(v) => {
                    self.bag
                        .insert(sym, fastn_compiler::LookupResult::Resolved(sym, v));
                }
                Err(v) => {
                    self.bag
                        .insert(sym, fastn_compiler::LookupResult::Unresolved(sym, v));
                }
            }
        }
    }

    async fn fetch_unresolved_symbols(
        &mut self,
        symbols_to_fetch: &mut [fastn_unresolved::SymbolName],
    ) {
        let symbols = self.symbols.lookup(&mut self.interner, symbols_to_fetch);
        for symbol in symbols {
            self.bag.insert(symbol.symbol(), symbol);
        }
    }

    /// try to resolve as many symbols as possible, and return the ones that we made any progress on.
    ///
    /// this function should be called in a loop, until the list of symbols is empty.
    fn resolve_symbols(
        &mut self,
        _symbols: &mut [fastn_unresolved::SymbolName],
    ) -> Vec<fastn_unresolved::Definition> {
        todo!()
    }

    /// try to make as much progress as possibly by resolving as many symbols as possible, and return
    /// the vec of ones that could not be resolved.
    ///
    /// if this returns an empty list of symbols, we can go ahead and generate the JS.
    fn resolve_document(&mut self) -> Vec<fastn_unresolved::SymbolName> {
        for ci in self.content.iter_mut() {
            if let fastn_unresolved::UR::UnResolved(_c) = ci {
                todo!()
            }
        }

        todo!()
    }

    async fn compile(&mut self) -> Result<fastn_compiler::Output, fastn_compiler::Error> {
        // we only make 10 attempts to resolve the document: we need a warning if we are not able to
        // resolve the document in 10 attempts.
        for _ in 1..10 {
            // resolve_document can internally run in parallel.
            let mut unresolved_symbols = self.resolve_document();
            if unresolved_symbols.is_empty() {
                break;
            }

            self.fetch_unresolved_symbols(&mut unresolved_symbols).await;
            // this itself has to happen in a loop. we need a warning if we are not able to resolve all
            // symbols in 10 attempts.
            for _ in 1..10 {
                // resolve_document can internally run in parallel.
                let partially_resolved = self.resolve_symbols(&mut unresolved_symbols);
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

/// this is our main compiler
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
    Compiler::new(symbols, auto_imports, document_id, source)
        .compile()
        .await
}
