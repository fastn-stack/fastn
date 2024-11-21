#[derive(Debug)]
pub struct UDF {
    pub name: String,
    pub params: Vec<String>,
    pub args: Vec<(String, fastn_js::SetPropertyValue)>,
    pub body: Vec<fastn_resolved::evalexpr::ExprNode>,
    pub is_external_js_present: bool,
}

pub fn udf_with_arguments(
    name: &str,
    body: Vec<fastn_resolved::evalexpr::ExprNode>,
    args: Vec<(String, fastn_js::SetPropertyValue)>,
    is_external_js_present: bool,
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
        is_external_js_present,
    })
}
