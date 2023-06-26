use fastn_js::UDFStatement;

fn space() -> pretty::RcDoc<'static> {
    pretty::RcDoc::space()
}

fn text(t: &str) -> pretty::RcDoc<'static> {
    pretty::RcDoc::text(t.to_string())
}

pub fn to_js(ast: &[fastn_js::Ast]) -> String {
    let mut w = Vec::new();
    let o = pretty::RcDoc::intersperse(ast.iter().map(|f| f.to_js()), space());
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
        text("let")
            .append(space())
            .append(text(&self.name))
            .append(space())
            .append(text("="))
            .append(space())
            .append(text("fastn_dom.createKernel("))
            .append(text(&format!("{},", self.parent.clone())))
            .append(space())
            .append(text(self.element_kind.to_js()))
            .append(text(");"))
    }
}

impl fastn_js::SetProperty {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        text(format!("{}.setProperty(", self.element_name).as_str())
            .append(text(format!("{},", self.kind.to_js()).as_str()))
            .append(space())
            .append(text(format!("{});", self.value.to_js()).as_str()))
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
            fastn_js::ComponentStatement::SetProperty(set_property) => set_property.to_js(),
            fastn_js::ComponentStatement::Done { component_name } => {
                text(&format!("{component_name}.done();"))
            }
        }
    }
}

fn func(
    name: &str,
    params: &[String],
    body: Vec<pretty::RcDoc<'static>>,
) -> pretty::RcDoc<'static> {
    text("function")
        .append(space())
        .append(text(name))
        .append(text("("))
        .append(
            pretty::RcDoc::intersperse(
                params.iter().map(|v| text(v.as_str())),
                text(",").append(space()),
            )
            .nest(4)
            .group(),
        )
        .append(text(")"))
        .append(pretty::RcDoc::softline_())
        .append(
            pretty::RcDoc::softline()
                .append(text("{"))
                .append(pretty::RcDoc::softline_())
                .append(
                    pretty::RcDoc::intersperse(body, pretty::RcDoc::softline())
                        .group()
                        .nest(4),
                )
                .append(pretty::RcDoc::softline_())
                .append(text("}"))
                .group(),
        )
}

impl fastn_js::Component {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        func(
            self.name.as_str(),
            &self.params,
            self.body.iter().map(|f| f.to_js()).collect(),
        )
    }
}

fn quote(s: &str) -> pretty::RcDoc<'static> {
    text("\"")
        .append(text(&s.replace('\n', "\\n")))
        .append(text("\""))
}

impl fastn_js::MutableVariable {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        text("let")
            .append(space())
            .append(text(&self.name))
            .append(space())
            .append(text("="))
            .append(space())
            .append(text("fastn.mutable("))
            .append(if self.is_quoted {
                quote(self.value.as_str())
            } else {
                text(&self.value)
            })
            .append(text(");"))
    }
}

impl fastn_js::StaticVariable {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        text("let")
            .append(space())
            .append(text(self.name.as_str()))
            .append(space())
            .append(text("="))
            .append(space())
            .append(if self.is_quoted {
                quote(self.value.as_str())
            } else {
                text(self.value.as_str())
            })
            .append(text(";"))
    }
}

impl fastn_js::UDF {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        func(
            &self.name,
            &self.params,
            self.body.iter().map(|f| f.to_js()).collect(),
        )
    }
}

fn binary(op: &str, left: &UDFStatement, right: &UDFStatement) -> pretty::RcDoc<'static> {
    left.to_js()
        .append(space())
        .append(text(op))
        .append(space())
        .append(right.to_js())
}

impl UDFStatement {
    fn to_js(&self) -> pretty::RcDoc<'static> {
        match self {
            UDFStatement::Integer { value } => text(&value.to_string()),
            UDFStatement::Decimal { value } => text(&value.to_string()),
            UDFStatement::Boolean { value } => text(&value.to_string()),
            UDFStatement::String { value } => quote(value.as_str()),
            UDFStatement::Return { value } => text("return")
                .append(space())
                .append(value.to_js())
                .append(text(";")),
            UDFStatement::VariableDeclaration { name, value } => text("let")
                .append(space())
                .append(text(name.as_str()))
                .append(space())
                .append(text("="))
                .append(space())
                .append(value.to_js())
                .append(text(";")),
            UDFStatement::VariableAssignment { name, value } => text(name.as_str())
                .append(space())
                .append(text("="))
                .append(space())
                .append(value.to_js())
                .append(text(";")),
            UDFStatement::Addition { left, right } => binary("+", left, right),
            UDFStatement::Subtraction { left, right } => binary("-", left, right),
            UDFStatement::Multiplication { left, right } => binary("*", left, right),
            UDFStatement::Division { left, right } => binary("/", left, right),
            UDFStatement::Exponentiation { left, right } => binary("**", left, right),
            UDFStatement::And { left, right } => binary("&&", left, right),
            UDFStatement::Or { left, right } => binary("||", left, right),
            UDFStatement::Not { value } => text("!").append(value.to_js()),
            UDFStatement::Parens { value } => text("(").append(value.to_js()).append(text(")")),
            UDFStatement::Variable { name } => text(name.as_str()),
            UDFStatement::Ternary {
                condition,
                then,
                otherwise,
            } => condition
                .to_js()
                .append(space())
                .append(text("?"))
                .append(space())
                .append(then.to_js())
                .append(space())
                .append(text(":"))
                .append(space())
                .append(otherwise.to_js()),
            UDFStatement::If {
                condition,
                then,
                otherwise,
            } => text("if")
                .append(space())
                .append(text("("))
                .append(condition.to_js())
                .append(text(")"))
                .append(space())
                .append(text("{"))
                .append(then.to_js())
                .append(text("}"))
                .append(space())
                .append(text("else"))
                .append(space())
                .append(text("{"))
                .append(otherwise.to_js())
                .append(text("}")),
            UDFStatement::Call { name, args } => text(name.as_str())
                .append(text("("))
                .append(
                    pretty::RcDoc::intersperse(
                        args.iter().map(|f| f.to_js()),
                        text(",").append(space()),
                    )
                    .group(),
                )
                .append(text(")")),
            UDFStatement::Block { .. } => todo!(),
        }
    }
}

#[cfg(test)]
#[track_caller]
pub fn e(f: fastn_js::Ast, s: &str) {
    let g = to_js(&vec![f]);
    println!("got: {}", g);
    println!("expected: {}", s);
    assert_eq!(g, s);
}

#[cfg(test)]
mod tests {
    #[test]
    fn udf() {
        fastn_js::to_js::e(fastn_js::udf0("foo", vec![]), "function foo() {}");
        fastn_js::to_js::e(fastn_js::udf1("foo", "p", vec![]), "function foo(p) {}");
        fastn_js::to_js::e(
            fastn_js::udf2("foo", "p", "q", vec![]),
            "function foo(p, q) {}",
        );

        fastn_js::to_js::e(
            fastn_js::udf0(
                "foo",
                vec![fastn_js::UDFStatement::Return {
                    value: Box::new(fastn_js::UDFStatement::Integer { value: 10 }),
                }],
            ),
            "function foo() {return 10;}",
        );
        fastn_js::to_js::e(
            fastn_js::udf0(
                "foo",
                vec![fastn_js::UDFStatement::Return {
                    value: Box::new(fastn_js::UDFStatement::Decimal { value: 10.1 }),
                }],
            ),
            "function foo() {return 10.1;}",
        );
        fastn_js::to_js::e(
            fastn_js::udf0(
                "foo",
                vec![fastn_js::UDFStatement::Return {
                    value: Box::new(fastn_js::UDFStatement::Boolean { value: true }),
                }],
            ),
            "function foo() {return true;}",
        );
        fastn_js::to_js::e(
            fastn_js::udf0(
                "foo",
                vec![fastn_js::UDFStatement::Return {
                    value: Box::new(fastn_js::UDFStatement::String {
                        value: "hello".to_string(),
                    }),
                }],
            ),
            r#"function foo() {return "hello";}"#,
        );
        fastn_js::to_js::e(
            fastn_js::udf0(
                "foo",
                vec![fastn_js::UDFStatement::Call {
                    name: "bar".to_string(),
                    args: vec![fastn_js::UDFStatement::String {
                        value: "hello".to_string(),
                    }],
                }],
            ),
            r#"function foo() {bar("hello")}"#,
        );
    }

    #[test]
    fn test_func() {
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![]),
            "function foo(parent) {}",
        );
        fastn_js::to_js::e(
            fastn_js::component1("foo", "p", vec![]),
            "function foo(parent, p) {}",
        );
        fastn_js::to_js::e(
            fastn_js::component2("foo", "p", "q", vec![]),
            "function foo(parent, p, q) {}",
        );
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
