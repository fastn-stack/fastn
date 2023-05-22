// struct Encoder {
//     types: std::collections::HashMap<(Vec<wasm_encoder::ValType>, wasm_encoder::ValType), u32>,
//     type_count: u32,
//     functions: wasm_encoder::FunctionSection,
//     exports: wasm_encoder::ExportSection,
//     module: wasm_encoder::Module,
// }

pub fn encode(_ast: Vec<fastn_wasm::Ast>) -> Vec<u8> {
    let mut module = wasm_encoder::Module::new();

    // let _type_count: u32 = 0;
    // let _ = std::collections::HashMap::new();

    let mut types = wasm_encoder::TypeSection::new();
    let params = vec![];
    let results = vec![];
    types.function(params, results);

    let params = vec![wasm_encoder::ValType::I32];
    let results = vec![];
    types.function(params, results);

    module.section(&types);


    let mut functions = wasm_encoder::FunctionSection::new();
    let type_index = 0;
    functions.function(type_index);
    functions.function(type_index);
    module.section(&functions);

    let mut exports = wasm_encoder::ExportSection::new();
    exports.export("main", wasm_encoder::ExportKind::Func, 1);
    module.section(&exports);

    let mut codes = wasm_encoder::CodeSection::new();

    let locals = vec![(1, wasm_encoder::ValType::I32)];
    let mut f = wasm_encoder::Function::new(locals);
    f.instruction(&wasm_encoder::Instruction::I32Const(42));
    f.instruction(&wasm_encoder::Instruction::LocalSet(0));
    f.instruction(&wasm_encoder::Instruction::End);
    codes.function(&f);

    let locals = vec![(1, wasm_encoder::ValType::I32)];
    let mut f = wasm_encoder::Function::new(locals);
    f.instruction(&wasm_encoder::Instruction::I32Const(42));
    f.instruction(&wasm_encoder::Instruction::LocalSet(0));
    f.instruction(&wasm_encoder::Instruction::End);

    codes.function(&f);

    module.section(&codes);


    let bytes = module.finish();
    // ~/Downloads/wabt-1.0.33/bin/wasm2wat test.wasm
    std::fs::write("test.wasm", &bytes).unwrap();
    bytes
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
