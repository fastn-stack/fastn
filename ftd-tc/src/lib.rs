#![allow(unused)]

extern crate self as ftd_tc;

mod error;
mod parser;
mod types;

pub use error::*;
pub use parser::parse_document_to_ast;
pub use types::*;

impl ContinuableThing {
    pub fn from_component_invocation(
        c: ftd_ast::ComponentInvocation,
        document_id: DocumentID,
    ) -> Self {
        ContinuableThing::CI(CI {
            inner: c,
            local_types: ftd_p1::Map::new(),
            js_buffer: String::new(),
            to_resolve: vec![ComponentResolvable::Name],
            document_id,
        })
    }
}

impl State {
    fn merge_ast(
        &mut self,
        extend_continuable_things: bool,
        ast: Vec<ftd_ast::Ast>,
        doc_id: DocumentID,
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
                        Lined {
                            line_number: ast.line_number(),
                            v: ast,
                            doc_id: doc_id.clone(),
                        },
                    );
                }
                ftd_ast::Ast::VariableInvocation(_) => unreachable!(),
                ftd_ast::Ast::ComponentInvocation(c) => {
                    if extend_continuable_things {
                        self.continuable_things
                            .push(ContinuableThing::from_component_invocation(
                                c.clone(),
                                doc_id.clone(),
                            ))
                    }
                }
            }
        }
    }

    pub fn from_document(source: &str, doc_id: DocumentID) -> ftd_tc::Result<Self> {
        let ast = parse_document_to_ast(source, &doc_id)?;

        let mut s = Self::default();
        s.merge_ast(false, ast, doc_id);

        Ok(s)
    }

    fn handle_ci_thing(
        &mut self,
        c: &mut CI,
        thing: &mut ComponentResolvable,
    ) -> ftd_tc::Result<Option<String>> {
        match thing {
            ComponentResolvable::Name => {
                // see if name exists in self.global_types, if so move on to verifying
                // other stuff. if the name doesn't exist in global types, and belongs to
                // another module, and the module is already loaded, we move to CD state,
                // component definition. If the module is also not yet loaded we return
                // module name to load.
                match self.global_types.get(&c.inner.name) {
                    Some(Qualified {
                        v: Type::Component(c),
                        ..
                    }) => {
                        todo!()
                    }
                    Some(t) => todo!("syntax error, foo is not a component"),
                    None => match self.symbols.get(&c.inner.name) {
                        Some(Lined {
                            v: ftd_ast::Ast::ComponentDefinition(c),
                            ..
                        }) => {
                            todo!()
                        }
                        Some(t) => {
                            todo!("syntax error, foo is not a component")
                        }
                        None => todo!(),
                    },
                }
            }
            _ => Ok(None),
        }
    }

    fn resolve_component_invocation(&mut self, c: &mut CI) -> ftd_tc::Result<Option<String>> {
        while let Some(mut thing) = c.to_resolve.pop() {
            if let Some(document) = self.handle_ci_thing(c, &mut thing)? {
                c.to_resolve.push(thing);
                return Ok(Some(document));
            }
        }

        Ok(None)
    }

    fn resolve_record_invocation(&mut self, c: &mut RI) -> ftd_tc::Result<Option<String>> {
        Ok(None)
    }

    fn resolve_function_invocation(&mut self, c: &mut FI) -> ftd_tc::Result<Option<String>> {
        Ok(None)
    }

    fn handle_thing(&mut self, thing: &mut ContinuableThing) -> ftd_tc::Result<Option<String>> {
        match thing {
            ContinuableThing::CI(c) => self.resolve_component_invocation(c),
            ContinuableThing::RI(r) => self.resolve_record_invocation(r),
            ContinuableThing::FI(f) => self.resolve_function_invocation(f),
        }
    }

    fn r#continue(mut self) -> ftd_tc::Result<TCState> {
        while let Some(mut thing) = self.continuable_things.pop() {
            if let Some(document) = self.handle_thing(&mut thing)? {
                self.continuable_things.push(thing);
                return Ok(TCState::StuckOnImport {
                    document,
                    state: self,
                });
            }
        }

        Ok(TCState::Done(self))
    }

    fn continue_after_import(
        mut self,
        doc_id: DocumentID,
        source: &str,
    ) -> ftd_tc::Result<TCState> {
        let ast = parse_document_to_ast(source, &doc_id)?;
        self.merge_ast(false, ast, doc_id);
        self.r#continue()
    }
}

impl DocumentID {
    pub fn new(logical: &str, physical: &str) -> Self {
        Self {
            logical: logical.to_string(),
            physical: physical.to_string(),
        }
    }

    pub fn new0(logical: &str) -> Self {
        Self {
            logical: logical.to_string(),
            physical: logical.to_string(),
        }
    }
}
