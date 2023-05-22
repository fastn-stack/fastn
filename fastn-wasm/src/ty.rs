#[derive(Debug)]
pub enum Type {
    I32,
    I64,
    F32,
    F64,
    ExternRef,
    Void,
    FuncRef,
    EmptyBlockType,
}

impl Type {
    pub fn to_wat(&self) -> &'static str {
        match self {
            Type::I32 => "i32",
            Type::I64 => "i64",
            Type::F32 => "f32",
            Type::F64 => "f64",
            Type::ExternRef => "externref",
            Type::Void => "void",
            Type::FuncRef => "funcref",
            Type::EmptyBlockType => "empty_block_type",
        }
    }
}
