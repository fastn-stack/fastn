#[derive(Debug)]
pub struct Memory {
    pub limits: fastn_wasm::Limits,
    pub shared: bool,
}

impl Memory {
    pub fn to_doc(&self) -> pretty::RcDoc<'static> {
        todo!()
    }

    pub fn to_wat(&self) -> String {
        let limits_wat = self.limits.to_wat();
        let shared = if self.shared {
            " shared".to_string()
        } else {
            String::new()
        };
        format!("(memory {}{})", limits_wat, shared)
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

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(
            fastn_wasm::Memory {
                shared: false,
                limits: fastn_wasm::Limits { min: 2, max: None },
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (memory 2)
                )
            "#
            )
        );
    }
}
