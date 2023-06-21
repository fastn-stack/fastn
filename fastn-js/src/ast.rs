pub enum Ast {
    Component(fastn_js::Component),
    UDF(fastn_js::UDF), // user defined function
}

impl Ast {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        match self {
            Ast::Component(f) => f.to_js(),
            Ast::UDF(f) => f.to_js(),
        }
    }
}
