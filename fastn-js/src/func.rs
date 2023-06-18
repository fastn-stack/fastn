pub struct Func {
    name: String,
    params: Vec<String>,
    pub body: Vec<fastn_js::Instruction>,
}

pub fn func0(name: &str, body: Vec<fastn_js::Instruction>) -> Func {
    Func {
        name: name.to_string(),
        params: vec!["parent".to_string()],
        body,
    }
}

pub fn func1(name: &str, arg1: &str, body: Vec<fastn_js::Instruction>) -> Func {
    Func {
        name: name.to_string(),
        params: vec!["parent".to_string(), arg1.to_string()],
        body,
    }
}

pub fn func2(name: &str, arg1: &str, arg2: &str, body: Vec<fastn_js::Instruction>) -> Func {
    Func {
        name: name.to_string(),
        params: vec!["parent".to_string(), arg1.to_string(), arg2.to_string()],
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
                    pretty::RcDoc::text(",").append(pretty::RcDoc::space()),
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
#[track_caller]
pub fn e(f: fastn_js::Func, s: &str) {
    let g = fastn_js::encode(&vec![f]);
    println!("got: {}", g);
    println!("expected: {}", s);
    assert_eq!(g, s);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_func() {
        let func = fastn_js::func0("foo", vec![]);
        fastn_js::func::e(func, "function foo(parent) {}");
        let func = fastn_js::func1("foo", "p", vec![]);
        fastn_js::func::e(func, "function foo(parent, p) {}");
        let func = fastn_js::func2("foo", "p", "q", vec![]);
        fastn_js::func::e(func, "function foo(parent, p, q) {}");
    }
}
