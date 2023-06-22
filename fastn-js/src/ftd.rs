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

impl fastn_js::ComponentStatement {
    pub fn from_component(
        component: &ftd::interpreter::Component,
        parent: &str,
        index: usize,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        if fastn_js::utils::is_kernel(component.name.as_str()) {
            let kernel = fastn_js::Kernel::from_component(component.name.as_str(), parent, index);
            component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
            component_statements.push(fastn_js::ComponentStatement::Done {
                component_name: kernel.name,
            });
        } else {
            todo!()
        }
        component_statements
    }
}
