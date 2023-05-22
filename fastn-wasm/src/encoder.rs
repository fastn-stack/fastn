pub fn encode(_ast: Vec<fastn_wasm::Ast>) -> Vec<u8> {
    let mut module = wasm_encoder::Module::new();

    let mut types = wasm_encoder::TypeSection::new();
    let params = vec![];
    let results = vec![];
    types.function(params, results);
    module.section(&types);

    let mut functions = wasm_encoder::FunctionSection::new();
    let type_index = 0;
    functions.function(type_index);
    module.section(&functions);

    let mut exports = wasm_encoder::ExportSection::new();
    exports.export("main", wasm_encoder::ExportKind::Func, 0);
    module.section(&exports);

    let mut codes = wasm_encoder::CodeSection::new();
    let locals = vec![];
    let mut f = wasm_encoder::Function::new(locals);
    f.instruction(&wasm_encoder::Instruction::End);
    codes.function(&f);

    module.section(&codes);

    module.finish()
}

// impl ftd::wasm::Ast {
//     pub fn to_wat(&self) -> String {
//         match self {
//             ftd::wasm::Ast::Func(f) => f.to_wat(),
//         }
//     }
// }
//
// impl ftd::wasm::Func {
//     pub fn to_wat(&self) -> String {
//     }
// }