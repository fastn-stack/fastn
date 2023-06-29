pub struct MutableVariable {
    pub name: String,
    pub value: String,
    pub is_quoted: bool,
}

pub fn mutable_unquoted(name: &str, value: &str) -> fastn_js::ComponentStatement {
    fastn_js::ComponentStatement::MutableVariable(MutableVariable {
        name: name.to_string(),
        value: value.to_string(),
        is_quoted: false,
    })
}

pub fn mutable_quoted(name: &str, value: &str) -> fastn_js::ComponentStatement {
    fastn_js::ComponentStatement::MutableVariable(MutableVariable {
        name: name.to_string(),
        value: value.to_string(),
        is_quoted: true,
    })
}

pub struct MutableList {
    pub name: String,
    pub value: fastn_js::SetPropertyValue,
}
