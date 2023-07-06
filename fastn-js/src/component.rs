pub struct Component {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<fastn_js::ComponentStatement>,
}

pub fn component0(name: &str, body: Vec<fastn_js::ComponentStatement>) -> fastn_js::Ast {
    fastn_js::Ast::Component(Component {
        name: name.to_string(),
        params: vec!["parent".to_string(), "inherited".to_string()],
        body,
    })
}

pub fn component_with_params(
    name: &str,
    body: Vec<fastn_js::ComponentStatement>,
    params: Vec<String>,
) -> fastn_js::Ast {
    use itertools::Itertools;

    fastn_js::Ast::Component(Component {
        name: name.to_string(),
        params: [vec!["parent".to_string(), "inherited".to_string()], params]
            .concat()
            .into_iter()
            .map(|v| fastn_js::utils::name_to_js(v.as_str()))
            .collect_vec(),
        body,
    })
}

pub fn component1(
    name: &str,
    arg1: &str,
    body: Vec<fastn_js::ComponentStatement>,
) -> fastn_js::Ast {
    fastn_js::Ast::Component(Component {
        name: name.to_string(),
        params: vec!["parent".to_string(), arg1.to_string()],
        body,
    })
}

pub fn component2(
    name: &str,
    arg1: &str,
    arg2: &str,
    body: Vec<fastn_js::ComponentStatement>,
) -> fastn_js::Ast {
    fastn_js::Ast::Component(Component {
        name: name.to_string(),
        params: vec!["parent".to_string(), arg1.to_string(), arg2.to_string()],
        body,
    })
}
