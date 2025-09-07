const ITERATION_THRESHOLD: usize = 100;

// foo.ftd
// -- import: foo as f (f => foo)
//
// -- import: bar      (bar => Module<bar>, x => Symbol<bar.y>) (bar => bar, x => bar.y)
// exposing: y as x
pub struct Compiler {
    pub(crate) definitions_used: std::collections::HashSet<fastn_section::Symbol>,
    pub arena: fastn_section::Arena,
    pub(crate) definitions: std::collections::HashMap<String, fastn_unresolved::Urd>,
    /// checkout resolve_document for why this is an Option
    pub(crate) content: Option<Vec<fastn_unresolved::Urci>>,
    pub(crate) document: fastn_unresolved::Document,
    // pub global_aliases: fastn_unresolved::AliasesSimple,
    iterations: usize,
    pub main_package: fastn_package::MainPackage,
}

impl Compiler {
    fn new(
        source: &str,
        main_package: fastn_package::MainPackage,
        module: Option<&str>,
        // global_aliases: fastn_unresolved::AliasesSimple,
    ) -> Self {
        let mut arena = fastn_section::Arena::default();

        let mut document = fastn_unresolved::parse(
            &main_package,
            fastn_section::Module::new(main_package.name.as_str(), module, &mut arena),
            source,
            &mut arena,
            // &global_aliases,
        );
        let content = Some(document.content);
        document.content = vec![];

        Self {
            main_package,
            arena,
            definitions: std::collections::HashMap::new(),
            content,
            document,
            // global_aliases,
            definitions_used: Default::default(),
            iterations: 0,
        }
    }

    /// try to resolve as many symbols as possible, and return the ones that we made any progress on.
    ///
    /// this function should be called in a loop, until the list of symbols is empty.
    #[tracing::instrument(skip(self))]
    fn resolve_symbols(
        &mut self,
        symbols: std::collections::HashSet<fastn_section::Symbol>,
    ) -> ResolveSymbolsResult {
        let mut r = ResolveSymbolsResult::default();
        for symbol in symbols {
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
            let mut definition = self.definitions.remove(symbol.str(&self.arena));
            match definition.as_mut() {
                Some(fastn_unresolved::UR::UnResolved(definition)) => {
                    let mut o = Default::default();
                    definition.resolve(
                        &self.definitions,
                        &mut self.arena,
                        &mut o,
                        &self.main_package,
                    );
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
                        self.definitions.insert(
                            symbol.string(&self.arena),
                            fastn_unresolved::UR::Resolved(Some(resolved)),
                        );
                    }
                    Err(s) => {
                        r.need_more_symbols.insert(symbol.clone());
                        self.definitions.insert(
                            symbol.string(&self.arena),
                            fastn_unresolved::UR::UnResolved(*s),
                        );
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
    #[tracing::instrument(skip(self))]
    fn resolve_document(&mut self) -> std::collections::HashSet<fastn_section::Symbol> {
        let mut stuck_on_symbols = std::collections::HashSet::new();

        let content = self.content.replace(vec![]).unwrap();
        let mut new_content = vec![];

        for mut ci in content {
            let resolved = if let fastn_unresolved::UR::UnResolved(ref mut c) = ci {
                let mut needed = Default::default();
                let resolved = c.resolve(
                    &self.definitions,
                    &mut self.arena,
                    &mut needed,
                    &self.main_package,
                );
                stuck_on_symbols.extend(needed.stuck_on);
                self.document
                    .merge(needed.errors, needed.warnings, needed.comments);
                resolved
            } else {
                false
            };
            if resolved {
                ci.resolve_it(&self.arena)
            }
            new_content.push(ci);
        }
        self.content = Some(new_content);

        stuck_on_symbols
    }

    fn finalise(
        self,
        some_symbols_are_unresolved: bool,
    ) -> Result<fastn_resolved::CompiledDocument, fastn_compiler::Error> {
        // we are here means ideally we are done.
        // we could have some unresolvable symbols or self.document.errors may not be empty.
        if some_symbols_are_unresolved || !self.document.errors.is_empty() {
            // we were not able to resolve all symbols or there were errors
            // return Err(fastn_compiler::Error {
            //     messages: todo!(),
            //     resolved: todo!(),
            //     symbol_errors: todo!(),
            // });
            todo!();
        }

        // there were no errors, etc.
        Ok(fastn_resolved::CompiledDocument {
            content: fastn_compiler::utils::resolved_content(self.content.unwrap()),
            definitions: fastn_compiler::utils::used_definitions(
                self.definitions,
                self.definitions_used,
                self.arena,
            ),
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
#[tracing::instrument]
pub fn compile(
    source: &str,
    main_package: fastn_package::MainPackage,
    module: Option<&str>,
) -> fastn_continuation::Result<Compiler> {
    use fastn_continuation::Continuation;

    Compiler::new(source, main_package, module).continue_after(vec![])
}

impl fastn_continuation::Continuation for Compiler {
    type Output = Result<fastn_resolved::CompiledDocument, fastn_compiler::Error>;
    type Needed = std::collections::HashSet<fastn_section::Symbol>;
    type Found = Vec<fastn_unresolved::Urd>;

    #[tracing::instrument(skip(self))]
    fn continue_after(mut self, definitions: Self::Found) -> fastn_continuation::Result<Self> {
        self.iterations += 1;
        if self.iterations > ITERATION_THRESHOLD {
            panic!("iterations too high");
        }

        for definition in definitions {
            // the following is only okay if our symbol store only returns unresolved definitions,
            // some other store might return resolved definitions, and we need to handle that.
            self.definitions.insert(
                definition
                    .unresolved()
                    .unwrap()
                    .symbol
                    .clone()
                    .unwrap()
                    .string(&self.arena),
                definition,
            );
        }

        let unresolved_symbols = self.resolve_document();
        if unresolved_symbols.is_empty() {
            return fastn_continuation::Result::Done(self.finalise(false));
        }

        // this itself has to happen in a loop. we need a warning if we are not able to resolve all
        // symbols in 10 attempts.
        let mut r = ResolveSymbolsResult {
            need_more_symbols: unresolved_symbols,
            unresolvable: Default::default(),
        };
        r = self.resolve_symbols(r.need_more_symbols);

        if r.need_more_symbols.is_empty() {
            return fastn_continuation::Result::Done(self.finalise(true));
        }

        fastn_continuation::Result::Stuck(Box::new(self), r.need_more_symbols)
    }
}

#[derive(Default)]
struct ResolveSymbolsResult {
    need_more_symbols: std::collections::HashSet<fastn_section::Symbol>,
    unresolvable: std::collections::HashSet<fastn_section::Symbol>,
}
