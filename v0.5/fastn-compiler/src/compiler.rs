const ITERATION_THRESHOLD: usize = 100;

pub(crate) struct Compiler {
    symbols: Box<dyn fastn_compiler::SymbolStore>,
    symbols_used: std::collections::HashSet<fastn_unresolved::SymbolName>,
    interner: string_interner::DefaultStringInterner,
    bag: std::collections::HashMap<string_interner::DefaultSymbol, fastn_unresolved::LookupResult>,
    auto_imports: Vec<fastn_section::AutoImport>,
    /// checkout resolve_document for why this is an Option
    content: Option<
        Vec<
            fastn_unresolved::UR<
                fastn_unresolved::ComponentInvocation,
                fastn_type::ComponentInvocation,
            >,
        >,
    >,
    document: fastn_unresolved::Document,
}

impl Compiler {
    fn resolution_input(&self) -> fastn_unresolved::ResolutionInput {
        fastn_unresolved::ResolutionInput {
            bag: &self.bag,
            auto_imports: &self.auto_imports,
            builtins: &fastn_compiler::BUILTINS,
        }
    }

    fn new(
        symbols: Box<dyn fastn_compiler::SymbolStore>,
        auto_imports: Vec<fastn_section::AutoImport>,
        document_id: &fastn_unresolved::ModuleName,
        source: &str,
    ) -> Self {
        let mut document = fastn_unresolved::parse(document_id, source);
        let content = Some(document.content);
        document.content = vec![];

        Self {
            symbols,
            interner: string_interner::StringInterner::new(),
            bag: std::collections::HashMap::new(),
            content,
            auto_imports,
            document,
            symbols_used: Default::default(),
        }
    }

    fn fetch_unresolved_symbols(
        &mut self,
        symbols_to_fetch: &std::collections::HashSet<fastn_unresolved::SymbolName>,
    ) {
        self.symbols_used.extend(symbols_to_fetch.iter().cloned());
        let definitions = self.symbols.lookup(&mut self.interner, symbols_to_fetch);
        for definition in definitions {
            // the following is only okay if our symbol store only returns unresolved definitions,
            // some other store might return resolved definitions, and we need to handle that.
            self.bag
                .insert(definition.unresolved().unwrap().symbol.unwrap(), definition);
        }
    }

    /// try to resolve as many symbols as possible, and return the ones that we made any progress on.
    ///
    /// this function should be called in a loop, until the list of symbols is empty.
    fn resolve_symbols(
        &mut self,
        symbols: std::collections::HashSet<fastn_unresolved::SymbolName>,
    ) -> ResolveSymbolsResult {
        let mut r = ResolveSymbolsResult::default();
        for symbol in symbols {
            let sym = symbol.symbol(&mut self.interner);
            // what if this is a recursive definition?
            // `foo` calling `foo`?
            // we will not find `foo` in the `bag` anymore, so we have to explicitly check for that.
            // but what if `foo` calls `bar` and `bar` calls `foo`?
            // we will not be able to resolve that.
            // it won't be a problem because `definition.resolve()` is not recursive, meaning if
            // `foo` is being resolved,
            // and it encounters `bar`, we will not try to internally
            // resolve `bar`, we will stop till bar is fully resolved.
            // in case of recursion, the foo will have first resolved its signature, and then,
            // when `bar` needs signature of `foo,`
            // it will find it from the partially resolved
            // `foo` in the `bag`.
            // to make sure this happens better, we have to ensure that the definition.resolve()
            // tries to resolve the signature first, and then the body.
            let mut definition = self.bag.remove(&sym);
            match definition.as_mut() {
                Some(fastn_unresolved::UR::UnResolved(definition)) => {
                    let o = definition.resolve(self.resolution_input());
                    r.need_more_symbols.extend(o.stuck_on);
                    self.document.merge(o.errors, o.warnings, o.comments);
                }
                Some(fastn_unresolved::UR::Resolved(_)) => unreachable!(),
                _ => {
                    r.unresolvable.insert(symbol.clone());
                }
            }
            if let Some(fastn_unresolved::UR::UnResolved(definition)) = definition {
                match definition.resolved() {
                    Ok(resolved) => {
                        self.bag
                            .insert(sym, fastn_unresolved::UR::Resolved(resolved));
                    }
                    Err(s) => {
                        r.need_more_symbols.insert(symbol);
                        self.bag.insert(sym, fastn_unresolved::UR::UnResolved(s));
                    }
                }
            }
        }

        r
    }

    /// try to make as much progress as possibly by resolving as many symbols as possible, and return
    /// the vec of ones that could not be resolved.
    ///
    /// if this returns an empty list of symbols, we can go ahead and generate the JS.
    fn resolve_document(&mut self) -> std::collections::HashSet<fastn_unresolved::SymbolName> {
        let mut stuck_on_symbols = std::collections::HashSet::new();

        let content = self.content.replace(vec![]).unwrap();
        let mut new_content = vec![];

        for ci in content {
            match ci {
                fastn_unresolved::UR::UnResolved(mut c) => {
                    let needed = c.resolve(self.resolution_input());
                    if needed.stuck_on.is_empty() {
                        new_content.push(fastn_unresolved::UR::Resolved(c.resolved().unwrap()));
                    } else {
                        stuck_on_symbols.extend(needed.stuck_on);
                        new_content.push(fastn_unresolved::UR::UnResolved(c));
                    }
                    self.document
                        .merge(needed.errors, needed.warnings, needed.comments);
                }
                v => new_content.push(v),
            }
        }

        self.content = Some(new_content);

        stuck_on_symbols
    }

    fn compile(mut self) -> Result<fastn_compiler::Output, fastn_compiler::Error> {
        // we only make 10 attempts to resolve the document: we need a warning if we are not able to
        // resolve the document in 10 attempts.
        let mut unresolvable = std::collections::HashSet::new();
        // let mut ever_used = std::collections::HashSet::new();
        let mut iterations = 0;
        while iterations < ITERATION_THRESHOLD {
            // resolve_document can internally run in parallel.
            // TODO: pass unresolvable to self.resolve_document() and make sure they don't come back
            let unresolved_symbols = self.resolve_document();
            if unresolved_symbols.is_empty() {
                break;
            }
            // ever_used.extend(&unresolved_symbols);
            self.fetch_unresolved_symbols(&unresolved_symbols);
            // this itself has to happen in a loop. we need a warning if we are not able to resolve all
            // symbols in 10 attempts.
            let mut r = ResolveSymbolsResult::default();
            r.need_more_symbols.extend(unresolved_symbols);

            while iterations < ITERATION_THRESHOLD {
                // resolve_document can internally run in parallel.
                // TODO: pass unresolvable to self.resolve_symbols() and make sure they don't come back
                r = self.resolve_symbols(r.need_more_symbols);
                unresolvable.extend(r.unresolvable);
                if r.need_more_symbols.is_empty() {
                    break;
                }
                // ever_used.extend(r.need_more_symbols);
                self.fetch_unresolved_symbols(&r.need_more_symbols);
                iterations += 1;
            }

            iterations += 1;
        }

        // we are here means ideally we are done.
        // we could have some unresolvable symbols or self.document.errors may not be empty.
        if !unresolvable.is_empty()
            || !self.document.errors.is_empty()
            || iterations == ITERATION_THRESHOLD
        {
            // we were not able to resolve all symbols or there were errors
            return Err(fastn_compiler::Error {
                messages: todo!(),
                resolved: todo!(),
                symbol_errors: todo!(),
            });
        }

        // there were no errors, etc.
        Ok(fastn_compiler::Output {
            js: self.js(),
            warnings: self.document.warnings,
            resolved: vec![], // for now, we are not tracking resolved
        })
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
pub fn compile(
    symbols: Box<dyn fastn_compiler::SymbolStore>,
    document_id: &fastn_unresolved::ModuleName,
    source: &str,
    auto_imports: Vec<fastn_section::AutoImport>,
) -> Result<fastn_compiler::Output, fastn_compiler::Error> {
    Compiler::new(symbols, auto_imports, document_id, source).compile()
}

#[derive(Default)]
struct ResolveSymbolsResult {
    need_more_symbols: std::collections::HashSet<fastn_unresolved::SymbolName>,
    unresolvable: std::collections::HashSet<fastn_unresolved::SymbolName>,
}
