#[derive(Debug)]
pub struct Component {
    pub name: String,
    pub params: Vec<String>,
    pub args: Vec<(String, fastn_js::SetPropertyValue, bool)>, // Vec<(name, value, is_mutable)>
    pub body: Vec<fastn_js::ComponentStatement>,
}

pub fn component0(name: &str, body: Vec<fastn_js::ComponentStatement>) -> fastn_js::Ast {
    fastn_js::Ast::Component(Component {
        name: name.to_string(),
        params: vec![fastn_js::COMPONENT_PARENT.to_string()],
        args: vec![],
        body,
    })
}

pub fn component_with_params(
    name: &str,
    body: Vec<fastn_js::ComponentStatement>,
    args: Vec<(String, fastn_js::SetPropertyValue, bool)>,
) -> fastn_js::Ast {
    fastn_js::Ast::Component(Component {
        name: name.to_string(),
        params: vec![
            fastn_js::COMPONENT_PARENT.to_string(),
            fastn_js::INHERITED_VARIABLE.to_string(),
            "args".to_string(),
        ],
        args,
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
        params: vec![fastn_js::COMPONENT_PARENT.to_string(), arg1.to_string()],
        args: vec![],
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
        params: vec![
            fastn_js::COMPONENT_PARENT.to_string(),
            arg1.to_string(),
            arg2.to_string(),
        ],
        args: vec![],
        body,
    })
}
