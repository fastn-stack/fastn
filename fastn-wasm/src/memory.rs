#[derive(Debug)]
pub struct Memory {
    pub limits: fastn_wasm::Limits,
    pub shared: bool,
}

impl Memory {
    pub fn to_doc(&self) -> pretty::RcDoc<'static> {
        let limits_wat = self.limits.to_wat();
        let shared = if self.shared {
            " shared".to_string()
        } else {
            String::new()
        };

        fastn_wasm::named(
            "memory",
            Some(pretty::RcDoc::text(format!("{}{}", limits_wat, shared))),
        )
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
}

#[cfg(test)]
mod test {
    #[track_caller]
    fn e(f: fastn_wasm::Memory, s: &str) {
        let g = fastn_wasm::encode_new(&vec![fastn_wasm::Ast::Memory(f)]);
        println!("got: {}", g);
        println!("expected: {}", s);
        assert_eq!(g, s);
    }

    #[test]
    fn test() {
        e(
            fastn_wasm::Memory {
                shared: false,
                limits: fastn_wasm::Limits { min: 2, max: None },
            },
            "(module (memory 2))",
        );
    }
}
