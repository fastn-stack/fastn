pub struct StaticVariable {
    pub name: String,
    pub value: String,
}

pub fn static_unquoted(name: &str, value: &str) -> fastn_js::Instruction {
    fastn_js::Instruction::StaticVariable(StaticVariable {
        name: name.to_string(),
        value: value.to_string(),
    })
}

impl StaticVariable {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        pretty::RcDoc::text("let")
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text(self.name.clone()))
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text("="))
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text(self.value.clone()))
            .append(pretty::RcDoc::text(";"))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_func2() {
        let func = fastn_js::func0("foo", vec![fastn_js::static_unquoted("bar", "10")]);
        fastn_js::func::e(
            func,
            indoc::indoc!(
                r#"
            function foo(parent) {let bar = 10;}"#,
            ),
        );
    }
}
