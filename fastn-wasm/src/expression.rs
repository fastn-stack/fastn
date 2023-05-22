#[derive(Debug)]
pub enum Expression {
    GlobalSet {
        index: Box<Expression>,
        value: Box<Expression>,
    },
    I32Const(i32),
    Call {
        name: String,
        params: Vec<Expression>,
    },
    Data {
        offset: i32,
        data: Vec<u8>,
    },
}

impl Expression {
    pub fn to_wat(&self) -> String {
        "todo".to_string()
    }
}
