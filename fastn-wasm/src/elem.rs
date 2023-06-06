#[derive(Debug)]
pub struct Elem {
    pub start: u32,
    pub fns: Vec<String>,
}

impl Elem {
    pub fn to_doc(&self) -> pretty::RcDoc<'static> {
        fastn_wasm::group(
            "elem".to_string(),
            Some(pretty::RcDoc::text(format!("(i32.const {})", self.start))),
            pretty::RcDoc::intersperse(
                self.fns
                    .iter()
                    .map(|v| pretty::RcDoc::text(format!("${}", v))),
                pretty::RcDoc::space(),
            ),
        )
    }
}

#[cfg(test)]
mod test {
    #[track_caller]
    fn e(f: super::Elem, s: &str) {
        let g = fastn_wasm::encode(&vec![fastn_wasm::Ast::Elem(f)]);
        println!("got: {}", g);
        println!("expected: {}", s);
        assert_eq!(g, s);
    }

    #[test]
    fn test() {
        e(
            super::Elem {
                start: 10,
                fns: vec!["f1".to_string(), "foo".to_string()],
            },
            "(module (elem (i32.const 10) $f1 $foo))",
        );
    }
}
