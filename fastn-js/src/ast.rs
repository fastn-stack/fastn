#[derive(Debug)]
pub enum Ast {
    Component(fastn_js::Component),
    UDF(fastn_js::UDF), // user defined function
    StaticVariable(fastn_js::StaticVariable),
    MutableVariable(fastn_js::MutableVariable),
    MutableList(fastn_js::MutableList),
    RecordInstance(fastn_js::RecordInstance),
    OrType(fastn_js::OrType),
    Export { from: String, to: String },
}
