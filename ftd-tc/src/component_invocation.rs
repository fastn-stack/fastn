use crate::{DocumentID, Lined, Qualified, Type};

#[derive(Debug)]
pub enum ComponentResolvable {
    /// First step is to resolve name, to see if there is any component with this name
    Name,
    /// Then we check the `id`. So far we are using `id` as literal, but if id was a reference
    /// we have to resolve and prove that reference is valid
    Id,
    Loop,
    Property(String),
    Argument(String),
    Event(String),
    Condition,
    Child(i32),
}

#[derive(Debug)]
pub struct CI {
    pub inner: ftd_ast::ComponentInvocation,
    pub to_resolve: Vec<ComponentResolvable>,
    pub local_types: ftd_p1::Map<Type>,
    pub js_buffer: String,
    pub document_id: DocumentID,
}

impl ftd_tc::State {
    pub fn handle_ci_name(
        &mut self,
        ci: &mut CI,
        thing: &mut ComponentResolvable,
    ) -> ftd_tc::Result<Option<String>> {
        // see if name exists in self.global_types, if so move on to verifying
        // other stuff. if the name doesn't exist in global types, and belongs to
        // another module, and the module is already loaded, we move to CD state,
        // component definition. If the module is also not yet loaded we return
        // module name to load.
        match self.global_types.get(&ci.inner.name) {
            Some(Qualified {
                v: Type::Component(c),
                ..
            }) => {
                // we are done with the first check, there is indeed a component with this name
                // next we have to check the `id`.
                ci.to_resolve.push(ComponentResolvable::Id);
                // TODO: check if the v is accessible in the current document
                Ok(None)
            }
            Some(t) => {
                self.errors.push(ftd_tc::Error::NotAComponent {
                    name: ci.inner.name.clone(),
                    usage_document: ci.document_id.clone(),
                    usage_line: ci.inner.line_number,
                    found: Box::new(t.to_owned()),
                });
                Ok(None)
            }
            None => match self.symbols.get(&ci.inner.name) {
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

    pub fn handle_ci_thing(
        &mut self,
        c: &mut CI,
        thing: &mut ComponentResolvable,
    ) -> ftd_tc::Result<Option<String>> {
        match thing {
            ComponentResolvable::Name => self.handle_ci_name(c, thing),
            ComponentResolvable::Id => todo!(),
            _ => Ok(None),
        }
    }

    pub fn resolve_component_invocation(&mut self, c: &mut CI) -> ftd_tc::Result<Option<String>> {
        while let Some(mut thing) = c.to_resolve.pop() {
            if let Some(document) = self.handle_ci_thing(c, &mut thing)? {
                c.to_resolve.push(thing);
                return Ok(Some(document));
            }
        }

        Ok(None)
    }
}

impl ftd_tc::ContinuableThing {
    pub fn from_component_invocation(
        c: ftd_ast::ComponentInvocation,
        document_id: ftd_tc::DocumentID,
    ) -> Self {
        ftd_tc::ContinuableThing::CI(ftd_tc::CI {
            inner: c,
            local_types: ftd_p1::Map::new(),
            js_buffer: String::new(),
            to_resolve: vec![ftd_tc::ComponentResolvable::Name],
            document_id,
        })
    }
}
