impl ftd::interpreter::Value {
    pub fn create(&self) -> ftd::wasm::Expression {
        match self {
            ftd::interpreter::Value::String {text} => {
                let data = text.as_bytes().to_vec();

                ftd::wasm::Expression::Call {
                    name: "string_new".to_string(),
                    params: vec![
                        ftd::wasm::Expression::I32Const(data.len() as i32),
                        ftd::wasm::Expression::Data {
                            offset: 0,
                            data
                        },
                    ],
                }
            }
            _ => panic!("Not implemented: {:?}", self),
        }
    }
}