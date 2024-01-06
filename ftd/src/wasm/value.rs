impl ftd::interpreter::Value {
    pub fn create(&self) -> fastn_wasm::Expression {
        match self {
            ftd::interpreter::Value::String { text } => {
                let data = text.into_bytes();

                fastn_wasm::Expression::Call {
                    name: "string_new".to_string(),
                    params: vec![
                        fastn_wasm::Expression::I32Const(data.len() as i32),
                        fastn_wasm::Expression::Data { offset: 0, data },
                    ],
                }
            }
            _ => panic!("Not implemented: {:?}", self),
        }
    }
}
