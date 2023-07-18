#[derive(Debug)]
pub struct UDF {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<fastn_grammar::evalexpr::ExprNode>,
}

pub fn udf0(name: &str, body: Vec<fastn_grammar::evalexpr::ExprNode>) -> fastn_js::Ast {
    fastn_js::Ast::UDF(UDF {
        name: name.to_string(),
        params: vec![],
        body,
    })
}

pub fn udf_with_params(
    name: &str,
    body: Vec<fastn_grammar::evalexpr::ExprNode>,
    params: Vec<String>,
) -> fastn_js::Ast {
    use itertools::Itertools;

    fastn_js::Ast::UDF(UDF {
        name: name.to_string(),
        params: params
            .into_iter()
            .map(|v| fastn_js::utils::name_to_js(v.as_str()))
            .collect_vec(),
        body,
    })
}

pub fn udf1(name: &str, arg1: &str, body: Vec<fastn_grammar::evalexpr::ExprNode>) -> fastn_js::Ast {
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
    body: Vec<fastn_grammar::evalexpr::ExprNode>,
) -> fastn_js::Ast {
    fastn_js::Ast::UDF(UDF {
        name: name.to_string(),
        params: vec![arg1.to_string(), arg2.to_string()],
        body,
    })
}
