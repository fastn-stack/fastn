pub struct Component {
    name: String,
    params: Vec<String>,
    pub body: Vec<fastn_js::Statement>,
}

pub fn component0(name: &str, body: Vec<fastn_js::Statement>) -> fastn_js::Ast {
    fastn_js::Ast::Component(Component {
        name: name.to_string(),
        params: vec!["parent".to_string()],
        body,
    })
}

pub fn component1(name: &str, arg1: &str, body: Vec<fastn_js::Statement>) -> fastn_js::Ast {
    fastn_js::Ast::Component(Component {
        name: name.to_string(),
        params: vec!["parent".to_string(), arg1.to_string()],
        body,
    })
}

pub fn component2(
    name: &str,
    arg1: &str,
    arg2: &str,
    body: Vec<fastn_js::Statement>,
) -> fastn_js::Ast {
    fastn_js::Ast::Component(Component {
        name: name.to_string(),
        params: vec!["parent".to_string(), arg1.to_string(), arg2.to_string()],
        body,
    })
}

impl Component {
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
        fastn_js::component::e(func, "function foo(parent) {}");
        let func = fastn_js::component1("foo", "p", vec![]);
        fastn_js::component::e(func, "function foo(parent, p) {}");
        let func = fastn_js::component2("foo", "p", "q", vec![]);
        fastn_js::component::e(func, "function foo(parent, p, q) {}");
    }
}
