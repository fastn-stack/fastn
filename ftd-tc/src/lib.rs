#![allow(unused)]

extern crate self as ftd_tc;

#[derive(Default)]
pub struct State {
    /// These are the things we need to resolve.
    ///
    /// we start by adding every component invocation in the main document and try to resolve
    /// them. If we find a reference to another document, we load that document and process it.
    /// We do this in a recursive manner.
    continuable_things: Vec<ContinuableThing>,
    /// Raw symbols from all documents are stored here
    symbols: ftd_p1::Map<ftd_ast::Ast>,
    /// any type we have already resolved is stored here
    global_types: ftd_p1::Map<Type>,
    /// js_buffer contains the generated JS when we resolve any symbol
    js_buffer: String,
}

enum ContinuableThing {
    RI(RI),
    CI(CI),
    FI(FI),
}

impl ContinuableThing {
    fn from_component_invocation(c: ftd_ast::ComponentInvocation) -> Self {
        ContinuableThing::CI(CI {
            inner: c,
            local_types: ftd_p1::Map::new(),
            js_buffer: String::new(),
            to_resolve: vec![ComponentResolvable::Name],
        })
    }
}

struct FI {
    //
}

struct RI {
    pub inner: ftd_ast::VariableDefinition,
    pub r: Record,
    pub current_field: i32,
}

enum ComponentResolvable {
    Name,
    Id,
    Loop,
    Property(String),
    Argument(String),
    Event(String),
    Condition,
    Child(i32),
}

struct CI {
    inner: ftd_ast::ComponentInvocation,
    to_resolve: Vec<ComponentResolvable>,
    local_types: ftd_p1::Map<Type>,
    js_buffer: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ast: {0}")]
    Ast(#[from] ftd_ast::Error),
}

type Result<T> = std::result::Result<T, Error>;

impl State {
    fn merge_ast(&mut self, extend_continuable_things: bool, ast: Vec<ftd_ast::Ast>, doc_id: &str) {
        for ast in ast {
            match ast {
                ftd_ast::Ast::Import(_)
                | ftd_ast::Ast::Record(_)
                | ftd_ast::Ast::OrType(_)
                | ftd_ast::Ast::VariableDefinition(_)
                | ftd_ast::Ast::ComponentDefinition(_)
                | ftd_ast::Ast::FunctionDefinition(_)
                | ftd_ast::Ast::WebComponentDefinition(_) => {
                    self.symbols.insert(format!("{doc_id}#{}", ast.name()), ast);
                }
                ftd_ast::Ast::VariableInvocation(_) => unreachable!(),
                ftd_ast::Ast::ComponentInvocation(c) => {
                    if extend_continuable_things {
                        self.continuable_things
                            .push(ContinuableThing::from_component_invocation(c.clone()))
                    }
                }
            }
        }
    }

    pub fn from_document(source: &str, doc_id: &str) -> ftd_tc::Result<Self> {
        let ast = parse_document_to_ast(source, doc_id)?;

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
                    Some(Type::Component(c)) => {
                        todo!()
                    }
                    Some(t) => todo!(),
                    None => match self.symbols.get(&c.inner.name) {
                        Some(ftd_ast::Ast::ComponentDefinition(c)) => {
                            todo!()
                        }
                        Some(t) => {
                            todo!()
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

    fn start(mut self) -> ftd_tc::Result<TCState> {
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

    fn continue_after_import(mut self, doc_id: &str, source: &str) -> ftd_tc::Result<TCState> {
        let ast = parse_document_to_ast(source, doc_id)?;
        self.merge_ast(false, ast, doc_id);
        self.start()
    }
}

enum TCState {
    Processing(State),
    StuckOnImport { document: String, state: State },
    Done(State),
}

enum Type {
    Integer(VarMeta),
    MutableInteger(VarMeta),
    Record(Record),
    Component(Component),
}

struct VarMeta {
    /// if the field has a default value, we can skip passing this field in the invocation
    has_default: bool,
    /// we can show the precise line number where the field is defined
    line: usize,
}

/// we use field to model component arguments, record fields, and function arguments etc
struct Field {
    name: String,
    type_: Type,
    meta: VarMeta,
}

struct Component {
    args: Vec<Field>,
}

struct Record {
    fields: Vec<Field>,
}

pub fn parse_document_to_ast(source: &str, doc_id: &str) -> ftd_ast::Result<Vec<ftd_ast::Ast>> {
    let sections = ftd_p1::parse(source, doc_id)?;
    let ast = ftd_ast::Ast::from_sections(sections.as_slice(), doc_id)?;
    println!("{:?}", ast);

    Ok(ast)
}
