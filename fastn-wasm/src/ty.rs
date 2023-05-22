#[derive(Debug)]
pub enum Type {
    I32,
    I64,
    F32,
    F64,
}

impl Type {
    pub fn to_wat(&self) -> &'static str {
        match self {
            Type::I32 => "i32",
            Type::I64 => "i64",
            Type::F32 => "f32",
            Type::F64 => "f64",
        }
    }
}
