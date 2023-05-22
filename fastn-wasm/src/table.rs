#[derive(Debug)]
pub struct Table {
    pub ref_type: fastn_wasm::RefType,
    pub limits: fastn_wasm::Limits,
}

impl Table {
    pub fn to_wat(&self) -> String {
        let limits_wat = self.limits.to_wat();
        let ref_type_wat = self.ref_type.to_wat();
        format!("(table {} {})", limits_wat, ref_type_wat)
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
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

impl Limits {
    pub fn to_wat(&self) -> String {
        let min_wat = self.min.to_string();
        let max_wat = self
            .max
            .map(|max| format!(" {}", max))
            .unwrap_or(String::new());
        format!("{}{}", min_wat, max_wat)
    }
}

#[derive(Debug)]
pub enum RefType {
    FuncRef,
    ExternRef,
}

impl RefType {
    pub fn to_wat(&self) -> &str {
        match self {
            RefType::FuncRef => "funcref",
            RefType::ExternRef => "externref",
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(
            fastn_wasm::Table {
                ref_type: fastn_wasm::RefType::FuncRef,
                limits: fastn_wasm::Limits { min: 2, max: None },
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (table 2 funcref)
                )
            "#
            )
        );
        assert_eq!(
            fastn_wasm::Table {
                ref_type: fastn_wasm::RefType::FuncRef,
                limits: fastn_wasm::Limits {
                    min: 2,
                    max: Some(5)
                },
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (table 2 5 funcref)
                )
            "#
            )
        );
    }
}
