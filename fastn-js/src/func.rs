pub struct Func {
    name: String,
    params: Vec<String>,
    pub body: Vec<fastn_js::Instruction>,
}

pub fn func0(name: &str, body: Vec<fastn_js::Instruction>) -> Func {
    Func {
        name: name.to_string(),
        params: vec![],
        body,
    }
}

impl Func {
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
                    pretty::RcDoc::text(","),
                )
                .nest(2)
                .group(),
            )
            .append(pretty::RcDoc::text(")"))
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text("{}"))
    }
}

#[cfg(test)]
mod tests {
    #[track_caller]
    fn e(f: fastn_js::Func, s: &str) {
        let g = fastn_js::encode(&vec![f]);
        println!("got: {}", g);
        println!("expected: {}", s);
        assert_eq!(g, s);
    }

    #[test]
    fn test_func() {
        let func = fastn_js::func0("foo", vec![]);
        e(func, "function foo() {}");
    }
}
