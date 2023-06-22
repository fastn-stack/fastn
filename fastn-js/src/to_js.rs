pub fn to_js(ast: &[fastn_js::Ast]) -> String {
    let mut w = Vec::new();
    let o = pretty::RcDoc::intersperse(ast.iter().map(|f| f.to_js()), pretty::RcDoc::space());
    o.render(80, &mut w).unwrap();
    String::from_utf8(w).unwrap()
}

impl fastn_js::Ast {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        match self {
            fastn_js::Ast::Component(f) => f.to_js(),
            fastn_js::Ast::UDF(f) => f.to_js(),
        }
    }
}

impl fastn_js::Kernel {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        pretty::RcDoc::text("let")
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text(self.name.clone()))
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text("="))
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text("fastn_dom.createKernel("))
            .append(pretty::RcDoc::text(format!("{},", self.parent.clone())))
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text(self.element_kind.to_js()))
            .append(pretty::RcDoc::text(");"))
    }
}

impl fastn_js::ElementKind {
    pub fn to_js(&self) -> &'static str {
        match self {
            fastn_js::ElementKind::Row => "fastn_dom.ElementKind.Row",
            fastn_js::ElementKind::Column => "fastn_dom.ElementKind.Column",
            fastn_js::ElementKind::Integer => "fastn_dom.ElementKind.Integer",
            fastn_js::ElementKind::Decimal => "fastn_dom.ElementKind.Decimal",
            fastn_js::ElementKind::Boolean => "fastn_dom.ElementKind.Boolean",
            fastn_js::ElementKind::Text => "fastn_dom.ElementKind.Text",
            fastn_js::ElementKind::Image => "fastn_dom.ElementKind.Image",
            fastn_js::ElementKind::IFrame => "fastn_dom.ElementKind.IFrame",
        }
    }
}
impl fastn_js::ComponentStatement {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        match self {
            fastn_js::ComponentStatement::StaticVariable(f) => f.to_js(),
            fastn_js::ComponentStatement::MutableVariable(f) => f.to_js(),
            fastn_js::ComponentStatement::CreateKernel(kernel) => kernel.to_js(),
            fastn_js::ComponentStatement::Done { component_name } => {
                pretty::RcDoc::text(format!("{component_name}.done();"))
            }
        }
    }
}

impl fastn_js::Component {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        pretty::RcDoc::text("function")
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text(self.name.clone()))
            .append(pretty::RcDoc::text("("))
            .append(
                pretty::RcDoc::intersperse(
                    self.params
                        .iter()
                        .map(|v| pretty::RcDoc::text(v.to_string())),
                    pretty::RcDoc::text(",").append(pretty::RcDoc::space()),
                )
                .nest(4)
                .group(),
            )
            .append(pretty::RcDoc::text(")"))
            .append(pretty::RcDoc::softline_())
            .append(
                pretty::RcDoc::softline()
                    .append(pretty::RcDoc::text("{"))
                    .append(pretty::RcDoc::softline_())
                    .append(
                        pretty::RcDoc::intersperse(
                            self.body.iter().map(|v| v.to_js()),
                            pretty::RcDoc::softline(),
                        )
                        .group()
                        .nest(4),
                    )
                    .append(pretty::RcDoc::softline_())
                    .append(pretty::RcDoc::text("}"))
                    .group(),
            )
    }
}

impl fastn_js::MutableVariable {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        pretty::RcDoc::text("let")
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text(self.name.clone()))
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text("="))
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text("fastn.mutable("))
            .append(if self.is_quoted {
                pretty::RcDoc::text("\"")
                    .append(pretty::RcDoc::text(self.value.replace('\n', "\\n")))
                    .append(pretty::RcDoc::text("\""))
            } else {
                pretty::RcDoc::text(self.value.clone())
            })
            .append(pretty::RcDoc::text(");"))
    }
}

impl fastn_js::StaticVariable {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        pretty::RcDoc::text("let")
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text(self.name.clone()))
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text("="))
            .append(pretty::RcDoc::space())
            .append(if self.is_quoted {
                pretty::RcDoc::text("\"")
                    .append(pretty::RcDoc::text(self.value.replace('\n', "\\n")))
                    .append(pretty::RcDoc::text("\""))
            } else {
                pretty::RcDoc::text(self.value.clone())
            })
            .append(pretty::RcDoc::text(";"))
    }
}

impl fastn_js::UDF {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        todo!()
    }
}

#[cfg(test)]
#[track_caller]
pub fn e(f: fastn_js::Ast, s: &str) {
    let g = fastn_js::to_js(&vec![f]);
    println!("got: {}", g);
    println!("expected: {}", s);
    assert_eq!(g, s);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_func() {
        let func = fastn_js::component0("foo", vec![]);
        fastn_js::to_js::e(func, "function foo(parent) {}");
        let func = fastn_js::component1("foo", "p", vec![]);
        fastn_js::to_js::e(func, "function foo(parent, p) {}");
        let func = fastn_js::component2("foo", "p", "q", vec![]);
        fastn_js::to_js::e(func, "function foo(parent, p, q) {}");
    }

    #[test]
    fn unquoted() {
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::mutable_unquoted("bar", "10")]),
            r#"function foo(parent) {let bar = fastn.mutable(10);}"#,
        );
    }

    #[test]
    fn quoted() {
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::mutable_quoted("bar", "10")]),
            r#"function foo(parent) {let bar = fastn.mutable("10");}"#,
        );
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::mutable_quoted("bar", "hello world")]),
            r#"function foo(parent) {let bar = fastn.mutable("hello world");}"#,
        );
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::mutable_quoted("bar", "hello world, a long long long long long string which keeps going on and on and on and on till we run out of line space and still keeps going on and on")]),
            indoc::indoc!(
                r#"function foo(parent) {
                let bar = fastn.mutable("hello world, a long long long long long string which keeps going on and on and on and on till we run out of line space and still keeps going on and on");
                }"#),
        );
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::mutable_quoted("bar", "hello\nworld")]),
            r#"function foo(parent) {let bar = fastn.mutable("hello\nworld");}"#,
        );
        // std::fs::write(
        //     "test.js",
        //     r#"function foo(parent) {let bar = "hello\nworld";}"#,
        // )
        // .unwrap();
    }

    #[test]
    fn static_unquoted() {
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::static_unquoted("bar", "10")]),
            r#"function foo(parent) {let bar = 10;}"#,
        );
    }

    #[test]
    fn static_quoted() {
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::static_quoted("bar", "10")]),
            r#"function foo(parent) {let bar = "10";}"#,
        );
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::static_quoted("bar", "hello world")]),
            r#"function foo(parent) {let bar = "hello world";}"#,
        );
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::static_quoted("bar", "hello world, a long long long long long string which keeps going on and on and on and on till we run out of line space and still keeps going on and on")]),
            indoc::indoc!(
                r#"function foo(parent) {
                let bar = "hello world, a long long long long long string which keeps going on and on and on and on till we run out of line space and still keeps going on and on";
                }"#),
        );
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::static_quoted("bar", "hello\nworld")]),
            r#"function foo(parent) {let bar = "hello\nworld";}"#,
        );
        // std::fs::write(
        //     "test.js",
        //     r#"function foo(parent) {let bar = "hello\nworld";}"#,
        // )
        // .unwrap();
    }
}
