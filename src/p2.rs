pub fn from_p1(p1: &[crate::p1::Section]) -> Result<Vec<Statement>, Error> {
    let mut stmts = vec![];
    for section in p1 {
        stmts.push(Statement::from_p1(section)?);
    }
    Ok(stmts)
}

pub enum Error {}

pub enum Statement {
    Function(Function),
    Enum(Enum),
    Const(Const),
    UsingLib(UsingLib),
}

impl Statement {
    fn from_p1(p1: &crate::p1::Section) -> Result<Statement, Error> {
        Ok(match p1.name.as_str() {
            "enum" => Statement::Enum(Enum::from_p1(p1)?),
            "const" => Statement::Const(Const::from_p1(p1)?),
            "using" => Statement::UsingLib(UsingLib::from_p1(p1)?),
            _ => Statement::Function(Function::from_p1(p1)?),
        })
    }
}

pub struct UsingLib(String);

impl UsingLib {
    fn from_p1(_p1: &crate::p1::Section) -> Result<Self, Error> {
        todo!()
    }
}

pub enum Const {
    String(String, String),
    Int(String, i32),
    Float(String, f32),
    Bool(String, bool),
    Element(String), // ID of some element
}

impl Const {
    fn from_p1(_p1: &crate::p1::Section) -> Result<Self, Error> {
        todo!()
    }
}

pub enum Enum {
    String {
        name: String,
        default: Option<String>,
        values: Vec<(String, String)>,
    },
    Int {
        name: String,
        default: Option<i32>,
        values: Vec<(String, i32)>,
    },
    Float {
        name: String,
        default: Option<f32>,
        values: Vec<(String, f32)>,
    },
}

impl Enum {
    fn from_p1(_p1: &crate::p1::Section) -> Result<Self, Error> {
        todo!()
    }
}

pub struct Function {
    pub id: String,
    pub root: String,
    pub arguments: Vec<Argument>,
    pub properties: Vec<Property>,
    pub children: Vec<Reference>,
    pub sub_functions: Vec<SubFunction>,
}

pub struct SubFunction {
    pub id: String,
    pub root: String,
    pub properties: Vec<Property>,
    pub children: Vec<Reference>,
}

impl Function {
    fn from_p1(_p1: &crate::p1::Section) -> Result<Self, Error> {
        todo!()
    }
}

pub enum Reference {
    ID(String),       // to a function without arguments
    Argument(String), // type_ must be either Element or Elements
}

pub struct Property {
    pub name: String,
    pub value: Value,
}

pub enum Value {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Argument(String),
}

pub struct Argument {
    pub name: String,
    pub type_: Type,
}

pub enum Type {
    String(Option<String>),
    Int(Option<i32>),
    Float(Option<f32>),
    Bool(Option<bool>),
    Element,
    Elements,
    Msg,  // just a message
    SMsg, // message that takes a string
    IMsg, // message that takes a int
}
