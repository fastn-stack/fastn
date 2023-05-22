#[derive(Debug)]
pub struct Import {
    pub module: String,
    pub name: String,
    pub desc: fastn_wasm::ImportDesc,
}

impl Import {
    pub fn to_wat(&self) -> String {
        let module_wat = &self.module;
        let name_wat = &self.name;
        let desc_wat = self.desc.to_wat();
        format!("(import \"{}\" \"{}\" {})", module_wat, name_wat, desc_wat)
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
pub enum ImportDesc {
    Func(fastn_wasm::FuncDecl),
}

impl ImportDesc {
    pub fn to_wat(&self) -> String {
        match self {
            ImportDesc::Func(f) => f.to_wat(),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(
            fastn_wasm::Import {
                module: "fastn".to_string(),
                name: "create_column".to_string(),
                desc: fastn_wasm::ImportDesc::Func(fastn_wasm::FuncDecl {
                    name: Some("create_column".to_string()),
                    params: vec![],
                    result: Some(fastn_wasm::Type::I32),
                }),
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (import "fastn" "create_column" (func $create_column (result i32)))
                )
            "#
            )
        );
    }
}
