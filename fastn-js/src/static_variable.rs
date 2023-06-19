pub struct StaticVariable {
    pub name: String,
    pub value: String,
    pub is_quoted: bool,
}

pub fn static_unquoted(name: &str, value: &str) -> fastn_js::Instruction {
    fastn_js::Instruction::StaticVariable(StaticVariable {
        name: name.to_string(),
        value: value.to_string(),
        is_quoted: false,
    })
}

pub fn static_quoted(name: &str, value: &str) -> fastn_js::Instruction {
    fastn_js::Instruction::StaticVariable(StaticVariable {
        name: name.to_string(),
        value: value.to_string(),
        is_quoted: true,
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
            .append(if self.is_quoted {
                pretty::RcDoc::text("\"")
                    .append(pretty::RcDoc::text(self.value.replace("\n", "\\n")))
                    .append(pretty::RcDoc::text("\""))
            } else {
                pretty::RcDoc::text(self.value.clone())
            })
            .append(pretty::RcDoc::text(";"))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn unquoted() {
        fastn_js::func::e(
            fastn_js::func0("foo", vec![fastn_js::static_unquoted("bar", "10")]),
            r#"function foo(parent) {let bar = 10;}"#,
        );
    }

    #[test]
    fn quoted() {
        fastn_js::func::e(
            fastn_js::func0("foo", vec![fastn_js::static_quoted("bar", "10")]),
            r#"function foo(parent) {let bar = "10";}"#,
        );
        fastn_js::func::e(
            fastn_js::func0("foo", vec![fastn_js::static_quoted("bar", "hello world")]),
            r#"function foo(parent) {let bar = "hello world";}"#,
        );
        fastn_js::func::e(
            fastn_js::func0("foo", vec![fastn_js::static_quoted("bar", "hello world, a long long long long long string which keeps going on and on and on and on till we run out of line space and still keeps going on and on")]),
            indoc::indoc!(
                r#"function foo(parent) {
                let bar = "hello world, a long long long long long string which keeps going on and on and on and on till we run out of line space and still keeps going on and on";
                }"#),
        );
        fastn_js::func::e(
            fastn_js::func0("foo", vec![fastn_js::static_quoted("bar", "hello\nworld")]),
            r#"function foo(parent) {let bar = "hello\nworld";}"#,
        );
        // std::fs::write(
        //     "test.js",
        //     r#"function foo(parent) {let bar = "hello\nworld";}"#,
        // )
        // .unwrap();
    }
}
