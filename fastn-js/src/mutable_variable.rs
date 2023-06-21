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

// https://github.community/t5/How-to-use-Git-and-GitHub/How-github-detect-trending-repositories/td-p/5925

#[cfg(test)]
mod tests {
    #[test]
    fn unquoted() {
        fastn_js::component::e(
            fastn_js::component0("foo", vec![fastn_js::mutable_unquoted("bar", "10")]),
            r#"function foo(parent) {let bar = fastn.mutable(10);}"#,
        );
    }

    #[test]
    fn quoted() {
        fastn_js::component::e(
            fastn_js::component0("foo", vec![fastn_js::mutable_quoted("bar", "10")]),
            r#"function foo(parent) {let bar = fastn.mutable("10");}"#,
        );
        fastn_js::component::e(
            fastn_js::component0("foo", vec![fastn_js::mutable_quoted("bar", "hello world")]),
            r#"function foo(parent) {let bar = fastn.mutable("hello world");}"#,
        );
        fastn_js::component::e(
            fastn_js::component0("foo", vec![fastn_js::mutable_quoted("bar", "hello world, a long long long long long string which keeps going on and on and on and on till we run out of line space and still keeps going on and on")]),
            indoc::indoc!(
                r#"function foo(parent) {
                let bar = fastn.mutable("hello world, a long long long long long string which keeps going on and on and on and on till we run out of line space and still keeps going on and on");
                }"#),
        );
        fastn_js::component::e(
            fastn_js::component0("foo", vec![fastn_js::mutable_quoted("bar", "hello\nworld")]),
            r#"function foo(parent) {let bar = fastn.mutable("hello\nworld");}"#,
        );
        // std::fs::write(
        //     "test.js",
        //     r#"function foo(parent) {let bar = "hello\nworld";}"#,
        // )
        // .unwrap();
    }
}
