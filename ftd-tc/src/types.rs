#[derive(Default, Debug)]
pub struct State {
    /// These are the things we need to resolve.
    ///
    /// we start by adding every component invocation in the main document and try to resolve
    /// them. If we find a reference to another document, we load that document and process it.
    /// We do this in a recursive manner.
    pub continuable_things: Vec<ContinuableThing>,
    /// Raw symbols from all documents are stored here
    pub symbols: ftd_p1::Map<Lined<ftd_ast::Ast>>,
    /// any type we have already resolved is stored here
    pub global_types: ftd_p1::Map<Qualified<Type>>,
    /// js_buffer contains the generated JS when we resolve any symbol
    pub js_buffer: String,
}

#[derive(Debug)]
pub enum ContinuableThing {
    RI(RI),
    CI(CI),
    FI(FI),
}

#[derive(Debug)]
pub struct FI {
    //
}

#[derive(Debug)]
pub struct RI {
    pub inner: ftd_ast::VariableDefinition,
    pub r: Record,
    pub current_field: i32,
}

#[derive(Debug)]
pub enum ComponentResolvable {
    Name,
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

#[derive(Debug)]
pub enum TCState {
    Processing(State),
    StuckOnImport { document: String, state: State },
    Done(State),
}

#[derive(Debug)]
pub enum Type {
    Integer,
    MutableInteger,
    Record(Record),
    Component(Component),
}

#[derive(Debug)]
pub enum AccessibleIn {
    /// accessible in the same document
    Module(DocumentID),
    /// accessible in the same package
    Package(DocumentID),
    /// accessible to anyone who adds this package as a direct dependency
    Public,
}

#[derive(Debug)]
pub struct Qualified<T> {
    pub v: T,
    pub line_number: usize,
    pub doc_id: DocumentID,
    pub accessible_in: AccessibleIn,
}

#[derive(Debug)]
pub struct Lined<T> {
    pub v: T,
    pub line_number: usize,
    pub doc_id: DocumentID,
}

#[derive(Debug, Clone)]
pub struct DocumentID {
    /// logical id is what we use to refer to a document in the code, eg `amitu.com/foo`
    pub logical: String,
    /// physical id is the file name, eg `.packages/amitu.com/foo/index.ftd`
    pub physical: String,
}

/// we use field to model component arguments, record fields, and function arguments etc
#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub type_: Type,
    /// if the field has a default value, we can skip passing this field in the invocation
    pub has_default: bool,
}

#[derive(Debug)]
pub struct Component {
    pub args: Vec<Field>,
}

#[derive(Debug)]
pub struct Record {
    pub fields: Vec<Field>,
}
