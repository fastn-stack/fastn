struct Encoder {
    types: std::collections::HashMap<(Vec<wasm_encoder::ValType>, wasm_encoder::ValType), u32>,
    type_count: u32,
    functions: wasm_encoder::FunctionSection,
    exports: wasm_encoder::ExportSection,
    module: wasm_encoder::Module,
}

impl Encoder {
    pub fn new() -> Self {
        Self {
            types: std::collections::HashMap::new(),
            type_count: 0,
            functions: wasm_encoder::FunctionSection::new(),
            exports: wasm_encoder::ExportSection::new(),
            module: wasm_encoder::Module::new(),
        }
    }

    pub fn encode_func(&mut self, func: fastn_wasm::Func) {
        let mut locals = vec![];
        for (name, ty) in func.locals {
            locals.push((1, wasm_encoder::ValType::I32));
        }
        let mut f = wasm_encoder::Function::new(locals);
        for ast in func.body {
            match ast {
                fastn_wasm::Ast::I32Const(i) => f.instruction(&wasm_encoder::Instruction::I32Const(i)),
                fastn_wasm::Ast::LocalSet(i) => f.instruction(&wasm_encoder::Instruction::LocalSet(i)),
                fastn_wasm::Ast::End => f.instruction(&wasm_encoder::Instruction::End),
                _ => todo!(),
            }
        }
        self.functions.function(0);
        self.module.section(&self.functions);
    }

    pub fn encode_ast(&mut self, ast: fastn_wasm::Ast) {
        match ast {
            fastn_wasm::Ast::Func(f) => self.encode_func(f),
        }
    }

    pub fn encode(ast: Vec<fastn_wasm::Ast>) -> Vec<u8> {
        let mut encoder = Self::new();
        for node in ast {
            encoder.encode_ast(node);
        }
        encoder.module.finish()
    }
}

pub fn encode(_ast: Vec<fastn_wasm::Ast>) -> Vec<u8> {

    let mut module = wasm_encoder::Module::new();


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

//
// impl ftd::wasm::Func {
//     pub fn to_wat(&self) -> String {
//     }
// }
