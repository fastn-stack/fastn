/// PL can be used for either Param or Local
#[derive(Debug, Clone)]
pub struct PL {
    pub name: Option<String>,
    pub ty: fastn_wasm::Type,
}

impl From<fastn_wasm::Type> for PL {
    fn from(ty: fastn_wasm::Type) -> Self {
        PL { name: None, ty }
    }
}

impl PL {
    pub fn to_doc(&self, is_param: bool) -> pretty::RcDoc<'static> {
        fastn_wasm::group(
            if is_param { "param" } else { "local" }.to_string(),
            self.name
                .clone()
                .map(|v| pretty::RcDoc::text(format!("${}", v))),
            self.ty.to_doc(),
        )
    }
}

#[cfg(test)]
mod test {
    #[track_caller]
    fn e(f: fastn_wasm::PL, is_param: bool, s: &str) {
        let mut w = Vec::new();
        let o = f.to_doc(is_param);
        o.render(80, &mut w).unwrap();
        let o = String::from_utf8(w).unwrap();
        println!("{}", o);

        println!("got: {}", o);
        println!("expected: {}", s);
        assert_eq!(o, s);
    }

    #[test]
    fn test() {
        e(
            fastn_wasm::PL {
                name: None,
                ty: fastn_wasm::Type::I32,
            },
            true,
            "(param i32)",
        );
        e(
            fastn_wasm::PL {
                name: None,
                ty: fastn_wasm::Type::I32,
            },
            false,
            "(local i32)",
        );
        e(
            fastn_wasm::PL {
                name: Some("foo".to_string()),
                ty: fastn_wasm::Type::I32,
            },
            true,
            "(param $foo i32)",
        );
        e(
            fastn_wasm::PL {
                name: Some("foo".to_string()),
                ty: fastn_wasm::Type::I32,
            },
            false,
            "(local $foo i32)",
        );
    }
}
