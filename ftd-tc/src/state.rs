#[derive(Default, Debug)]
pub struct State {
    /// These are the things we need to resolve.
    ///
    /// we start by adding every component invocation in the main document and try to resolve
    /// them. If we find a reference to another document, we load that document and process it.
    /// We do this in a recursive manner.
    pub continuable_things: Vec<ftd_tc::ContinuableThing>,
    /// Raw symbols from all documents are stored here
    pub symbols: ftd_p1::Map<ftd_tc::Lined<ftd_ast::Ast>>,
    /// any type we have already resolved is stored here
    pub global_types: ftd_p1::Map<ftd_tc::Qualified<ftd_tc::Type>>,
    /// js_buffer contains the generated JS when we resolve any symbol
    pub js_buffer: String,
}

#[derive(Debug)]
pub enum TCState {
    Processing(State),
    StuckOnImport { document: String, state: State },
    Done(State),
}

impl ftd_tc::State {
    fn merge_ast(
        &mut self,
        extend_continuable_things: bool,
        ast: Vec<ftd_ast::Ast>,
        doc_id: ftd_tc::DocumentID,
    ) {
        for ast in ast {
            match ast {
                ftd_ast::Ast::Import(_)
                | ftd_ast::Ast::Record(_)
                | ftd_ast::Ast::OrType(_)
                | ftd_ast::Ast::VariableDefinition(_)
                | ftd_ast::Ast::ComponentDefinition(_)
                | ftd_ast::Ast::FunctionDefinition(_)
                | ftd_ast::Ast::WebComponentDefinition(_) => {
                    self.symbols.insert(
                        format!("{}#{}", doc_id.logical, ast.name()),
                        ftd_tc::Lined {
                            line_number: ast.line_number(),
                            v: ast,
                            doc_id: doc_id.clone(),
                        },
                    );
                }
                ftd_ast::Ast::VariableInvocation(_) => unreachable!(),
                ftd_ast::Ast::ComponentInvocation(c) => {
                    if extend_continuable_things {
                        self.continuable_things.push(
                            ftd_tc::ContinuableThing::from_component_invocation(
                                c.clone(),
                                doc_id.clone(),
                            ),
                        )
                    }
                }
            }
        }
    }

    pub fn from_document(source: &str, doc_id: ftd_tc::DocumentID) -> ftd_tc::Result<Self> {
        let ast = ftd_tc::parse_document_to_ast(source, &doc_id)?;

        let mut s = Self::default();
        s.merge_ast(false, ast, doc_id);

        Ok(s)
    }

    fn resolve_record_invocation(&mut self, c: &mut ftd_tc::RI) -> ftd_tc::Result<Option<String>> {
        Ok(None)
    }

    fn resolve_function_invocation(
        &mut self,
        c: &mut ftd_tc::FI,
    ) -> ftd_tc::Result<Option<String>> {
        Ok(None)
    }

    fn handle_thing(
        &mut self,
        thing: &mut ftd_tc::ContinuableThing,
    ) -> ftd_tc::Result<Option<String>> {
        match thing {
            ftd_tc::ContinuableThing::CI(c) => self.resolve_component_invocation(c),
            ftd_tc::ContinuableThing::RI(r) => self.resolve_record_invocation(r),
            ftd_tc::ContinuableThing::FI(f) => self.resolve_function_invocation(f),
        }
    }

    fn r#continue(mut self) -> ftd_tc::Result<ftd_tc::TCState> {
        while let Some(mut thing) = self.continuable_things.pop() {
            if let Some(document) = self.handle_thing(&mut thing)? {
                self.continuable_things.push(thing);
                return Ok(ftd_tc::TCState::StuckOnImport {
                    document,
                    state: self,
                });
            }
        }

        Ok(ftd_tc::TCState::Done(self))
    }

    fn continue_after_import(
        mut self,
        doc_id: ftd_tc::DocumentID,
        source: &str,
    ) -> ftd_tc::Result<TCState> {
        let ast = ftd_tc::parse_document_to_ast(source, &doc_id)?;
        self.merge_ast(false, ast, doc_id);
        self.r#continue()
    }
}
