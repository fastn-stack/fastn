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

impl Ast {
    pub fn get_variable_name(&self) -> Option<String> {
        match self {
            Ast::StaticVariable(static_variable) => Some(static_variable.name.clone()),
            Ast::MutableVariable(mutable_variable) => Some(mutable_variable.name.clone()),
            Ast::RecordInstance(record_instance) => Some(record_instance.name.clone()),
            Ast::MutableList(mutable_list) => Some(mutable_list.name.clone()),
            _ => None,
        }
    }
}
