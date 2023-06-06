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
    pub fn to_doc(&self, is_param: bool) -> pretty::RcDoc<()> {
        fastn_wasm::group(
            if is_param { "param" } else { "local" },
            self.name
                .clone()
                .map(|v| pretty::RcDoc::text(format!("${}", v))),
            self.ty.to_doc(),
        )
    }

    pub fn to_wat(&self, is_param: bool) -> String {
        let mut s = String::new();
        if is_param {
            s.push_str("(param");
        } else {
            s.push_str("(local");
        }

        if let Some(name) = &self.name {
            s.push_str(" $");
            s.push_str(name);
        }
        s.push(' ');
        s.push_str(self.ty.to_wat());
        s.push(')');
        s
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(
            fastn_wasm::PL {
                name: None,
                ty: fastn_wasm::Type::I32,
            }
            .to_wat(true),
            "(param i32)"
        );
        assert_eq!(
            fastn_wasm::PL {
                name: None,
                ty: fastn_wasm::Type::I32,
            }
            .to_wat(false),
            "(local i32)"
        );
        assert_eq!(
            fastn_wasm::PL {
                name: Some("foo".to_string()),
                ty: fastn_wasm::Type::I32,
            }
            .to_wat(true),
            "(param $foo i32)"
        );
        assert_eq!(
            fastn_wasm::PL {
                name: Some("foo".to_string()),
                ty: fastn_wasm::Type::I32,
            }
            .to_wat(false),
            "(local $foo i32)"
        );
    }
}
