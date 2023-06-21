impl fastn_js::Ast {
    pub fn from_tree(tree: &[ftd::interpreter::Component]) -> fastn_js::Ast {
        fastn_js::Component::from_tree(tree)
    }
}

impl fastn_js::Component {
    pub fn from_tree(tree: &[ftd::interpreter::Component]) -> fastn_js::Ast {
        let mut statements = vec![];
        for (index, component) in tree.iter().enumerate() {
            statements.extend(fastn_js::ComponentStatement::from_component(
                component, "parent", index,
            ))
        }
        fastn_js::component0("main", statements)
    }
}
