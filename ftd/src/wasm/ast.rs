#[derive(Debug)]
pub enum Ast {
    Func(Func),
}

#[derive(Debug)]
pub struct Func {
    pub name: Option<String>,
    pub export: Option<String>,
    pub params: Vec<Param>,
    pub locals: Vec<Local>,
    pub body: Vec<Expression>,
}

#[derive(Debug)]
pub enum Type {
    I32,
    I64,
    F32,
    F64,
    ExternRef,
    FuncRef,
}

#[derive(Debug)]
pub struct Param {
    pub name: Option<String>,
    pub ty: Type,
}

#[derive(Debug)]
pub struct Local {
    pub name: Option<String>,
    pub ty: Type,
}

#[derive(Debug)]
pub enum Expression {
    GlobalSet { index: Box<Expression>, value: Box<Expression> },
    I32Const(i32),
    Call { name: String, params: Vec<Expression> },
    Data { offset: i32, data: Vec<u8> },
}