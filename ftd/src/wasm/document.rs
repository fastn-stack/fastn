impl ftd::interpreter::Document {
    pub fn generate_wasm(&self) -> Vec<fastn_wasm::Ast> {
        let mut globals = std::collections::HashMap::new();

        let mut wasm = vec![];
        // handle all global variables
        let mut main = fastn_wasm::Func {
            name: Some("main".to_string()),
            export: Some("main".to_string()),
            params: vec![],
            locals: vec![],
            body: vec![],
        };

        let mut global_index = 0usize;
        for thing in self.data.values() {
            if let ftd::interpreter::Thing::Variable(v) = thing {
                globals.insert(v.name.clone(), global_index);
                main.body.push(v.global_expression(global_index));
                global_index += 1;
            }
        }

        wasm.push(WasmAst::Func(main));

        wasm
    }
}
