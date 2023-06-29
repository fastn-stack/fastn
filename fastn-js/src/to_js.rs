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
            fastn_js::Ast::StaticVariable(s) => s.to_js(),
            fastn_js::Ast::MutableVariable(m) => m.to_js(),
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

impl fastn_js::EventHandler {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        text(format!("{}.addEventHandler(", self.element_name).as_str())
            .append(self.event.to_js())
            .append(text(","))
            .append(space())
            .append(text("function()"))
            .append(space())
            .append(text("{"))
            .append(self.action.to_js())
            .append(text("});"))
    }
}

impl fastn_js::Event {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        match self {
            fastn_js::Event::OnClick => text("fastn_dom.Event.Click"),
        }
    }
}

impl fastn_js::Function {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        text(format!("{}(", fastn_js::utils::name_to_js(self.name.as_str())).as_str())
            .append(pretty::RcDoc::intersperse(
                self.parameters.iter().map(|v| v.to_js()),
                text(",").append(space()),
            ))
            .append(text(");"))
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
            fastn_js::ComponentStatement::InstantiateComponent(i) => i.to_js(),
            fastn_js::ComponentStatement::AddEventHandler(e) => e.to_js(),
            fastn_js::ComponentStatement::Done { component_name } => {
                text(&format!("{component_name}.done();"))
            }
            fastn_js::ComponentStatement::Return { component_name } => {
                text(&format!("return {component_name};"))
            }
            fastn_js::ComponentStatement::ConditionalComponent(_) => todo!(),
        }
    }
}

impl fastn_js::InstantiateComponent {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        pretty::RcDoc::text(format!(
            "{}{}(",
            if self.should_return { "return " } else { "" },
            fastn_js::utils::name_to_js(self.name.as_str())
        ))
        .append(pretty::RcDoc::text(self.parent.clone()))
        .append(text(",").append(space()))
        .append(
            pretty::RcDoc::intersperse(
                self.arguments.iter().map(|v| v.to_js()),
                text(",").append(space()),
            )
            .group(),
        )
        .append(text(");"))
    }
}

fn func(
    name: &str,
    params: &[String],
    body: Vec<pretty::RcDoc<'static>>,
) -> pretty::RcDoc<'static> {
    text("function")
        .append(space())
        .append(text(fastn_js::utils::name_to_js(name).as_str()))
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
            .append(text(
                fastn_js::utils::name_to_js(self.name.as_str()).as_str(),
            ))
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
            .append(text(
                fastn_js::utils::name_to_js(self.name.as_str()).as_str(),
            ))
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
            self.body
                .iter()
                .map(|f| {
                    pretty::RcDoc::text(fastn_js::to_js::ExpressionGenerator.to_js_(
                        f,
                        true,
                        &self.params,
                    ))
                })
                .collect(),
        )
    }
}

/*fn binary(op: &str, left: &UDFStatement, right: &UDFStatement) -> pretty::RcDoc<'static> {
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
*/

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
    /*fn udf() {
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
    }*/
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

pub struct ExpressionGenerator;

impl ExpressionGenerator {
    pub fn to_js(&self, node: &fastn_grammar::evalexpr::ExprNode) -> String {
        self.to_js_(node, true, &[])
    }

    pub fn to_js_(
        &self,
        node: &fastn_grammar::evalexpr::ExprNode,
        root: bool,
        arguments: &[String],
    ) -> String {
        use itertools::Itertools;

        if self.is_root(node.operator()) {
            let result = node
                .children()
                .iter()
                .map(|children| self.to_js_(children, false, arguments))
                .collect_vec();
            let (is_assignment_or_chain, only_one_child) =
                node.children().first().map_or((false, true), |first| {
                    /*has_operator(dbg!(&first.operator())).is_none()*/
                    let is_assignment_or_chain =
                        self.is_assignment(first.operator()) || self.is_chain(first.operator());
                    (
                        is_assignment_or_chain,
                        is_assignment_or_chain
                            || self.has_value(first.operator()).is_some()
                            || self.is_tuple(first.operator()),
                    )
                });
            let f = if !only_one_child {
                format!("({})", result.join(""))
            } else {
                result.join("")
            };

            return if root && !is_assignment_or_chain && !f.is_empty() {
                format!("return {};", f)
            } else {
                f
            };
        }

        if self.is_chain(node.operator()) {
            let mut result = vec![];
            for children in node.children() {
                let val =
                    fastn_js::utils::trim_brackets(self.to_js_(children, true, arguments).trim());
                if !val.trim().is_empty() {
                    result.push(format!(
                        "{}{}",
                        val,
                        if val.ends_with(';') { "" } else { ";" }
                    ));
                }
            }
            return result.join("\n");
        }

        if self.is_tuple(node.operator()) {
            let mut result = vec![];
            for children in node.children() {
                result.push(self.to_js_(children, false, arguments));
            }
            return format!("[{}]", result.join(","));
        }

        if let Some(function_name) = self.function_name(node.operator()) {
            let mut result = vec![];
            if let Some(child) = node.children().first() {
                for children in child.children() {
                    let mut value = self.to_js_(children, false, arguments);
                    if self.is_tuple(children.operator()) {
                        value = value[1..value.len() - 1].to_string();
                    }
                    result.push(value);
                }
            }
            return format!("{}({})", function_name, result.join(","));
        }

        if self.is_assignment(node.operator()) {
            // Todo: if node.children().len() != 2 {throw error}
            let first = node.children().first().unwrap(); //todo remove unwrap()
            let second = node.children().get(1).unwrap(); //todo remove unwrap()
            if !arguments.iter().any(|v| first.to_string().eq(v)) {
                return vec![
                    "let ".to_string(),
                    self.to_js_(first, false, arguments),
                    node.operator().to_string(),
                    self.to_js_(second, false, arguments),
                ]
                .join("");
            } else if first.operator().get_variable_identifier_write().is_some() {
                let var = self.to_js_(first, false, arguments);
                let val = self.to_js_(second, false, arguments);
                return format!(
                    indoc::indoc! {
                        "let fastn_utils_val_{var} = {val};
                        if (!fastn_utils.setter({var}, fastn_utils_val_{var})) {{
                            {var} = fastn_utils_val_{var};
                        }}"
                    },
                    val = val,
                    var = var
                );
            };
            return vec![
                self.to_js_(first, false, arguments),
                node.operator().to_string(),
                self.to_js_(second, false, arguments),
            ]
            .join("");
        }

        if let Some(operator) = self.has_operator(node.operator()) {
            // Todo: if node.children().len() != 2 {throw error}
            let first = node.children().first().unwrap(); //todo remove unwrap()
            if matches!(node.operator(), fastn_grammar::evalexpr::Operator::Not)
                || matches!(node.operator(), fastn_grammar::evalexpr::Operator::Neg)
            {
                return vec![operator, self.to_js_(first, false, arguments)].join("");
            }
            let second = node.children().get(1).unwrap(); //todo remove unwrap()
            return vec![
                self.to_js_(first, false, arguments),
                operator,
                self.to_js_(second, false, arguments),
            ]
            .join("");
        }

        if let Some(operator) = self.has_function(node.operator()) {
            let mut result = vec![];
            for children in node.children() {
                result.push(self.to_js_(children, false, arguments));
            }
            return format!("{}{}", operator.trim(), result.join(" "));
        }

        let value = if self.is_null(node.operator()) {
            "null".to_string()
        } else {
            node.operator().to_string()
        };

        if node.operator().get_variable_identifier_read().is_some() {
            format!("fastn_utils.getter({})", value)
        } else {
            value
        }
    }

    pub fn has_value(&self, operator: &fastn_grammar::evalexpr::Operator) -> Option<String> {
        match operator {
            fastn_grammar::evalexpr::Operator::Const { .. }
            | fastn_grammar::evalexpr::Operator::VariableIdentifierRead { .. }
            | fastn_grammar::evalexpr::Operator::VariableIdentifierWrite { .. } => {
                Some(operator.to_string())
            }
            _ => None,
        }
    }

    pub fn has_function(&self, operator: &fastn_grammar::evalexpr::Operator) -> Option<String> {
        match operator {
            fastn_grammar::evalexpr::Operator::FunctionIdentifier { .. } => {
                Some(operator.to_string())
            }
            _ => None,
        }
    }

    pub fn is_assignment(&self, operator: &fastn_grammar::evalexpr::Operator) -> bool {
        matches!(operator, fastn_grammar::evalexpr::Operator::Assign)
    }

    pub fn is_chain(&self, operator: &fastn_grammar::evalexpr::Operator) -> bool {
        matches!(operator, fastn_grammar::evalexpr::Operator::Chain)
    }

    pub fn is_tuple(&self, operator: &fastn_grammar::evalexpr::Operator) -> bool {
        matches!(operator, fastn_grammar::evalexpr::Operator::Tuple)
    }

    pub fn is_null(&self, operator: &fastn_grammar::evalexpr::Operator) -> bool {
        matches!(
            operator,
            fastn_grammar::evalexpr::Operator::Const {
                value: fastn_grammar::evalexpr::Value::Empty,
            }
        )
    }

    pub fn function_name(&self, operator: &fastn_grammar::evalexpr::Operator) -> Option<String> {
        if let fastn_grammar::evalexpr::Operator::FunctionIdentifier { identifier } = operator {
            Some(identifier.to_string())
        } else {
            None
        }
    }

    pub fn has_operator(&self, operator: &fastn_grammar::evalexpr::Operator) -> Option<String> {
        if self.has_value(operator).is_none()
            && self.has_function(operator).is_none()
            && !self.is_chain(operator)
            && !self.is_root(operator)
            && !self.is_tuple(operator)
            && !self.is_assignment(operator)
        {
            Some(operator.to_string())
        } else {
            None
        }
    }

    pub fn is_root(&self, operator: &fastn_grammar::evalexpr::Operator) -> bool {
        matches!(operator, fastn_grammar::evalexpr::Operator::RootNode)
    }
}
