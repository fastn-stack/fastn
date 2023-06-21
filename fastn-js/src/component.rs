pub struct Component {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<fastn_js::ComponentStatement>,
}

pub fn component0(name: &str, body: Vec<fastn_js::ComponentStatement>) -> fastn_js::Ast {
    fastn_js::Ast::Component(Component {
        name: name.to_string(),
        params: vec!["parent".to_string()],
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
