pub struct StaticVariable {
    pub name: String,
    pub value: String,
    pub is_quoted: bool,
}

pub fn static_unquoted(name: &str, value: &str) -> fastn_js::ComponentStatement {
    fastn_js::ComponentStatement::StaticVariable(StaticVariable {
        name: name.to_string(),
        value: value.to_string(),
        is_quoted: false,
    })
}

pub fn static_quoted(name: &str, value: &str) -> fastn_js::ComponentStatement {
    fastn_js::ComponentStatement::StaticVariable(StaticVariable {
        name: name.to_string(),
        value: value.to_string(),
        is_quoted: true,
    })
}
