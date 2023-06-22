pub struct UDF {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<fastn_js::UDFStatement>,
}

pub fn udf0(name: &str, body: Vec<fastn_js::UDFStatement>) -> fastn_js::Ast {
    fastn_js::Ast::UDF(UDF {
        name: name.to_string(),
        params: vec![],
        body,
    })
}

pub fn udf1(name: &str, arg1: &str, body: Vec<fastn_js::UDFStatement>) -> fastn_js::Ast {
    fastn_js::Ast::UDF(UDF {
        name: name.to_string(),
        params: vec![arg1.to_string()],
        body,
    })
}

pub fn udf2(
    name: &str,
    arg1: &str,
    arg2: &str,
    body: Vec<fastn_js::UDFStatement>,
) -> fastn_js::Ast {
    fastn_js::Ast::UDF(UDF {
        name: name.to_string(),
        params: vec![arg1.to_string(), arg2.to_string()],
        body,
    })
}
