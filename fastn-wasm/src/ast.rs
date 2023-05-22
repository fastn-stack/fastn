#[derive(Debug)]
pub enum Ast {
    Func(Func),
}

#[derive(Debug)]
pub struct Func {
    pub name: Option<String>,
    pub export: Option<String>,
    pub params: Vec<PL>,
    pub locals: Vec<PL>,
    pub result: Option<wasm_encoder::ValType>,
    pub body: Vec<Expression>,
}


/// PL can be used for either Param or Local
#[derive(Debug)]
pub struct PL {
    pub name: Option<String>,
    pub ty: wasm_encoder::ValType,
}


#[derive(Debug)]
pub enum Expression {
    GlobalSet { index: Box<Expression>, value: Box<Expression> },
    I32Const(i32),
    Call { name: String, params: Vec<Expression> },
    Data { offset: i32, data: Vec<u8> },
}