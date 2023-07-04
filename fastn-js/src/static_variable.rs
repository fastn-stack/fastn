#[derive(Debug)]
pub struct StaticVariable {
    pub name: String,
    pub value: fastn_js::SetPropertyValue,
}

pub fn static_integer(name: &str, value: i64) -> fastn_js::ComponentStatement {
    fastn_js::ComponentStatement::StaticVariable(StaticVariable {
        name: name.to_string(),
        value: fastn_js::SetPropertyValue::Value(fastn_js::Value::Integer(value)),
    })
}

pub fn static_string(name: &str, value: &str) -> fastn_js::ComponentStatement {
    fastn_js::ComponentStatement::StaticVariable(StaticVariable {
        name: name.to_string(),
        value: fastn_js::SetPropertyValue::Value(fastn_js::Value::String(value.to_string())),
    })
}
