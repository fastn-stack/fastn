#[derive(Debug)]
pub struct Export {
    pub name: String,
    pub desc: fastn_wasm::ExportDesc,
}

pub fn func1(
    name: &str,
    arg0: fastn_wasm::PL,
    body: Vec<fastn_wasm::Expression>,
) -> fastn_wasm::Ast {
    fastn_wasm::Ast::Func(fastn_wasm::Func {
        export: Some(name.to_string()),
        params: vec![arg0],
        body,
        ..Default::default()
    })
}

impl Export {
    pub fn to_wat(&self) -> String {
        let desc_wat = self.desc.to_wat();
        format!("(export \"{}\" {})", self.name, desc_wat)
    }

    #[cfg(test)]
    pub fn to_wat_formatted(&self) -> String {
        wasmfmt::fmt(
            &self.to_wat(),
            wasmfmt::Options {
                resolve_names: false,
            },
        )
        .replace("\t", "    ")
    }
}

#[derive(Debug)]
pub enum ExportDesc {
    Func { index: fastn_wasm::Index },
}

impl ExportDesc {
    pub fn to_wat(&self) -> String {
        match self {
            ExportDesc::Func { index } => format!("(func {})", index.to_wat()),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(
            fastn_wasm::Export {
                name: "add".to_string(),
                desc: fastn_wasm::ExportDesc::Func {
                    index: "add".into()
                },
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (export "add" (func $add))
                )
            "#
            )
        );
    }
}
