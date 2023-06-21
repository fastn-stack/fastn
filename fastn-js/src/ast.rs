pub enum Ast {
    Component(fastn_js::Component),
    UDF(fastn_js::UDF), // user defined function
}
