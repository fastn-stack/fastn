impl ftd::interpreter::Variable {
    // Input:
    //  "foo#message", type: String, value: "Hello World"
    // Output:
    // (main (func $main (export "main")
    //      (global.set (i32.const idx) (call $string_new (i32.const 12) (data (i32.const 0) "Hello World")))
    // )
    pub fn global_expression(&self, idx: usize) -> fastn_wasm::Expression {
        let create = match &self.value {
            fastn_type::PropertyValue::Value { value, .. } => value.create(),
            _ => panic!("Not implemented: {:?}", self),
        };

        fastn_wasm::Expression::GlobalSet {
            index: Box::new(fastn_wasm::Expression::I32Const(idx as i32)),
            value: Box::new(create),
        }
    }
}
