#![allow(unused)]

extern crate self as ftd_tc;

pub struct State {
    /// These are the things we need to resolve.
    ///
    /// we start by adding every component invocation in the main document and try to resolve
    /// them. If we find a reference to another document, we load that document and process it.
    /// We do this in a recursive manner.
    continuable_things: Vec<ContinuableThing>,
    symbols: ftd_p1::Map<ftd_ast::Ast>,
    /// any type we resolve is stored here
    global_types: ftd_p1::Map<Type>,
    /// js_buffer contains the generated JS when we resolve any type
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
            to_resolve: todo!(),
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

enum ResolvableThing {
    Property(String),
    Event(String),
    Loop,
    Condition,
    Child(i32),
}

struct CI {
    inner: ftd_ast::ComponentInvocation,
    to_resolve: Vec<ResolvableThing>,
    local_types: ftd_p1::Map<Type>,
    js_buffer: String,
}

impl State {
    pub fn from_document(source: &str, doc_id: &str) -> ftd_ast::Result<Self> {
        let ast = parse_document_to_ast(source, doc_id)?;

        let mut continuable_things = vec![];
        let mut symbols = ftd_p1::Map::new();

        for ast in ast {
            match ast {
                ftd_ast::Ast::Import(_)
                | ftd_ast::Ast::Record(_)
                | ftd_ast::Ast::OrType(_)
                | ftd_ast::Ast::VariableDefinition(_)
                | ftd_ast::Ast::ComponentDefinition(_)
                | ftd_ast::Ast::FunctionDefinition(_)
                | ftd_ast::Ast::WebComponentDefinition(_) => {
                    symbols.insert(format!("{doc_id}#{}", ast.name()), ast);
                }
                ftd_ast::Ast::VariableInvocation(_) => unreachable!(),
                ftd_ast::Ast::ComponentInvocation(c) => {
                    continuable_things.push(ContinuableThing::from_component_invocation(c))
                }
            }
        }

        Ok(State {
            continuable_things,
            symbols,
            global_types: ftd_p1::Map::new(),
            js_buffer: String::new(),
        })
    }
}

enum TCState {
    Processing(State),
    StuckOnImport { document: String, state: State },
    Done(State),
}

// struct Sourced<T> {
//     file: String,
//     line: usize,
//     value: T,
// }

enum Type {
    Integer,
    MutableInteger,
    Record(Record),
}

struct Record {
    fields: Vec<(String, Type)>,
}

pub fn parse_document_to_ast(source: &str, doc_id: &str) -> ftd_ast::Result<Vec<ftd_ast::Ast>> {
    let sections = ftd_p1::parse(source, doc_id)?;
    let ast = ftd_ast::Ast::from_sections(sections.as_slice(), doc_id)?;
    println!("{:?}", ast);

    Ok(ast)
}
