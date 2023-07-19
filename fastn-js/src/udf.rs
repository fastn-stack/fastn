#[derive(Debug)]
pub struct UDF {
    pub name: String,
    pub params: Vec<String>,
    pub args: Vec<(String, fastn_js::SetPropertyValue)>,
    pub body: Vec<fastn_grammar::evalexpr::ExprNode>,
}

pub fn udf0(name: &str, body: Vec<fastn_grammar::evalexpr::ExprNode>) -> fastn_js::Ast {
    fastn_js::Ast::UDF(UDF {
        name: name.to_string(),
        params: vec!["args".to_string()],
        args: vec![],
        body,
    })
}

pub fn udf_with_arguments(
    name: &str,
    body: Vec<fastn_grammar::evalexpr::ExprNode>,
    args: Vec<(String, fastn_js::SetPropertyValue)>,
) -> fastn_js::Ast {
    use itertools::Itertools;

    fastn_js::Ast::UDF(UDF {
        name: name.to_string(),
        params: vec!["args".to_string()],
        args: args
            .into_iter()
            .map(|(key, val)| (fastn_js::utils::name_to_js(key.as_str()), val))
            .collect_vec(),
        body,
    })
}

pub fn udf1(name: &str, arg1: &str, body: Vec<fastn_grammar::evalexpr::ExprNode>) -> fastn_js::Ast {
    fastn_js::Ast::UDF(UDF {
        name: name.to_string(),
        params: vec![arg1.to_string()],
        args: vec![],
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
        args: vec![],
        body,
    })
}
