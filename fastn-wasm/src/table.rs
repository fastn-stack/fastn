#[derive(Debug)]
pub struct Table {
    pub ref_type: fastn_wasm::RefType,
    pub limits: fastn_wasm::Limits,
}

pub fn table(count: u32, ref_type: fastn_wasm::RefType) -> fastn_wasm::Ast {
    fastn_wasm::Ast::Table(Table {
        ref_type,
        limits: fastn_wasm::Limits {
            min: count,
            max: None,
        },
    })
}

pub fn table_1(ref_type: fastn_wasm::RefType, fn1: &str) -> Vec<fastn_wasm::Ast> {
    vec![
        table(1, ref_type),
        fastn_wasm::Ast::Elem(fastn_wasm::Elem {
            start: 0,
            fns: vec![fn1.to_string()],
        }),
    ]
}

pub fn table_2(ref_type: fastn_wasm::RefType, fn1: &str, fn2: &str) -> Vec<fastn_wasm::Ast> {
    vec![
        table(2, ref_type),
        fastn_wasm::Ast::Elem(fastn_wasm::Elem {
            start: 0,
            fns: vec![fn1.to_string(), fn2.to_string()],
        }),
    ]
}

pub fn table_3(
    ref_type: fastn_wasm::RefType,
    fn1: &str,
    fn2: &str,
    fn3: &str,
) -> Vec<fastn_wasm::Ast> {
    vec![
        table(3, ref_type),
        fastn_wasm::Ast::Elem(fastn_wasm::Elem {
            start: 0,
            fns: vec![fn1.to_string(), fn2.to_string(), fn3.to_string()],
        }),
    ]
}

pub fn table_4(
    ref_type: fastn_wasm::RefType,
    fn1: &str,
    fn2: &str,
    fn3: &str,
    fn4: &str,
) -> Vec<fastn_wasm::Ast> {
    vec![
        table(4, ref_type),
        fastn_wasm::Ast::Elem(fastn_wasm::Elem {
            start: 0,
            fns: vec![
                fn1.to_string(),
                fn2.to_string(),
                fn3.to_string(),
                fn4.to_string(),
            ],
        }),
    ]
}

impl Table {
    pub fn to_doc(&self) -> pretty::RcDoc<()> {
        todo!()
    }

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
    Func,
    Extern,
}

impl RefType {
    pub fn to_wat(&self) -> &str {
        match self {
            RefType::Func => "funcref",
            RefType::Extern => "externref",
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(
            fastn_wasm::Table {
                ref_type: fastn_wasm::RefType::Func,
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
                ref_type: fastn_wasm::RefType::Func,
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
