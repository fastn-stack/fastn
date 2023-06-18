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

#[cfg(test)]
mod tests {
    #[test]
    fn test_func2() {
        let func = fastn_js::func0("foo", vec![fastn_js::static_unquoted("bar", "10")]);
        fastn_js::func::e(
            func,
            indoc::indoc!(
                r#"
            function foo(parent) {
                let bar = 10;
            }"#,
            ),
        );
    }
}
