#[derive(Debug)]
pub enum ContinuableThing {
    RI(RI),
    CI(ftd_tc::CI),
    FI(FI),
}

#[derive(Debug)]
pub struct FI {
    //
}

#[derive(Debug, Clone)]
pub struct RI {
    pub inner: ftd_ast::VariableDefinition,
    pub r: Record,
    pub current_field: i32,
}

#[derive(Debug, Clone)]
pub enum Type {
    Integer,
    MutableInteger,
    Record(Record),
    Component(Component),
}

#[derive(Debug, Clone)]
pub enum AccessibleIn {
    /// accessible in the same document
    Module(DocumentID),
    /// accessible in the same package
    Package(DocumentID),
    /// accessible to anyone who adds this package as a direct dependency
    Public,
}

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub type_: Type,
    /// if the field has a default value, we can skip passing this field in the invocation
    pub has_default: bool,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub args: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Record {
    pub fields: Vec<Field>,
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
