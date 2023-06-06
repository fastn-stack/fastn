#[derive(Debug, Clone)]
pub enum Expression {
    GlobalSet {
        index: Index,
        value: Box<Expression>,
    },
    LocalSet {
        index: Index,
        value: Box<Expression>,
    },
    LocalGet {
        index: Index,
    },
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),
    Operation {
        name: String,
        values: Vec<Expression>,
    },
    Call {
        name: String,
        params: Vec<Expression>,
    },
    CallIndirect {
        type_: String,
        params: Vec<Expression>,
    },
    Data {
        offset: i32,
        data: Vec<u8>,
    },
    Drop,
}

pub fn call(name: &str) -> fastn_wasm::Expression {
    fastn_wasm::Expression::Call {
        name: name.into(),
        params: vec![],
    }
}

pub fn local(name: &str) -> fastn_wasm::Expression {
    fastn_wasm::Expression::LocalGet { index: name.into() }
}

pub fn local_set(name: &str, e: fastn_wasm::Expression) -> fastn_wasm::Expression {
    fastn_wasm::Expression::LocalSet {
        index: name.into(),
        value: Box::new(e),
    }
}

pub fn i32(i: i32) -> fastn_wasm::Expression {
    fastn_wasm::Expression::I32Const(i)
}

pub fn operation_2(
    op: &str,
    e0: fastn_wasm::Expression,
    e1: fastn_wasm::Expression,
) -> fastn_wasm::Expression {
    fastn_wasm::Expression::Operation {
        name: op.to_string(),
        values: vec![e0, e1],
    }
}

pub fn call_indirect2(
    type_: &str,
    e0: fastn_wasm::Expression,
    e1: fastn_wasm::Expression,
) -> fastn_wasm::Expression {
    fastn_wasm::Expression::CallIndirect {
        type_: type_.into(),
        params: vec![e0, e1],
    }
}

pub fn call1(name: &str, e0: fastn_wasm::Expression) -> fastn_wasm::Expression {
    fastn_wasm::Expression::Call {
        name: name.into(),
        params: vec![e0],
    }
}

pub fn call2(
    name: &str,
    e0: fastn_wasm::Expression,
    e1: fastn_wasm::Expression,
) -> fastn_wasm::Expression {
    fastn_wasm::Expression::Call {
        name: name.into(),
        params: vec![e0, e1],
    }
}

pub fn call3(
    name: &str,
    e0: fastn_wasm::Expression,
    e1: fastn_wasm::Expression,
    e2: fastn_wasm::Expression,
) -> fastn_wasm::Expression {
    fastn_wasm::Expression::Call {
        name: name.into(),
        params: vec![e0, e1, e2],
    }
}

pub fn call4(
    name: &str,
    e0: fastn_wasm::Expression,
    e1: fastn_wasm::Expression,
    e2: fastn_wasm::Expression,
    e3: fastn_wasm::Expression,
) -> fastn_wasm::Expression {
    fastn_wasm::Expression::Call {
        name: name.into(),
        params: vec![e0, e1, e2, e3],
    }
}

impl Expression {
    pub fn to_wat(&self) -> String {
        match self {
            Expression::GlobalSet { index, value } => {
                let index_wat = index;
                let value_wat = value.to_wat();
                format!("(global.set {} {})", index_wat.to_wat(), value_wat)
            }
            Expression::LocalSet { index, value } => {
                let index_wat = index;
                let value_wat = value.to_wat();
                format!("(local.set {} {})", index_wat.to_wat(), value_wat)
            }
            Expression::LocalGet { index } => {
                let index_wat = index;
                format!("(local.get {})", index_wat.to_wat())
            }
            Expression::I32Const(value) => format!("(i32.const {})", value),
            Expression::I64Const(value) => format!("(i64.const {})", value),
            Expression::F32Const(value) => format!("(f32.const {})", value),
            Expression::F64Const(value) => format!("(f64.const {})", value),
            Expression::Operation { name, values } => {
                let values_wat: Vec<String> = values.iter().map(|v| v.to_wat()).collect();
                format!("({} {})", name, values_wat.join(" "))
            }
            Expression::Call { name, params } => {
                let params_wat: Vec<String> = params.iter().map(|p| p.to_wat()).collect();
                format!("(call ${}{})", name, format!(" {}", params_wat.join(" ")))
            }
            Expression::CallIndirect { type_, params } => {
                let params_wat: Vec<String> = params.iter().map(|p| p.to_wat()).collect();
                format!("(call_indirect (type ${}) {})", type_, params_wat.join(" "))
            }
            Expression::Data { offset, data } => {
                let data_hex: Vec<String> = data.iter().map(|b| format!("{:02X}", b)).collect();
                format!("(data (i32.const {}) \"{}\")", offset, data_hex.join(""))
            }
            Expression::Drop => "(drop)".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Index {
    Index(i32),
    Variable(String),
}

impl From<i32> for Index {
    fn from(value: i32) -> Self {
        Index::Index(value)
    }
}

impl From<&str> for Index {
    fn from(value: &str) -> Self {
        Index::Variable(value.to_string())
    }
}

impl Index {
    pub fn to_wat(&self) -> String {
        match self {
            Index::Index(i) => i.to_string(),
            Index::Variable(v) => format!("${v}"),
        }
    }
}
