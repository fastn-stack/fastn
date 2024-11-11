pub enum Definition {
    Component(Component),
    Function(Function),
}

pub struct Component {}
pub struct Function {}

pub struct Document {
    pub definitions: Vec<Definition>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Expr {
    Integer(i32),
    String(String),
    Boolean(bool),
    Decimal(f32),
    Variable(String),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}
