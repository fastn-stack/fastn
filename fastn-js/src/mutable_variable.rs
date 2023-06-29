pub struct MutableVariable {
    pub name: String,
    pub value: fastn_js::SetPropertyValue,
}

pub fn mutable_integer(name: &str, value: i64) -> fastn_js::ComponentStatement {
    fastn_js::ComponentStatement::MutableVariable(MutableVariable {
        name: name.to_string(),
        value: fastn_js::SetPropertyValue::Value(fastn_js::Value::Integer(value)),
    })
}

pub fn mutable_string(name: &str, value: &str) -> fastn_js::ComponentStatement {
    fastn_js::ComponentStatement::MutableVariable(MutableVariable {
        name: name.to_string(),
        value: fastn_js::SetPropertyValue::Value(fastn_js::Value::String(value.to_string())),
    })
}

pub struct MutableList {
    pub name: String,
    pub value: fastn_js::SetPropertyValue,
}
