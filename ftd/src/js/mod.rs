#![allow(dead_code)]

#[cfg(test)]
#[macro_use]
mod ftd_test_helpers;
mod utils;

pub fn document_into_js_ast(document: ftd::interpreter::Document) -> Vec<fastn_js::Ast> {
    vec![ftd::js::from_tree(document.tree.as_slice())]
}

pub fn from_tree(tree: &[ftd::interpreter::Component]) -> fastn_js::Ast {
    let mut statements = vec![];
    for (index, component) in tree.iter().enumerate() {
        statements.extend(component.into_component_statements("parent", index))
    }
    fastn_js::component0("main", statements)
}

impl ftd::interpreter::Component {
    pub fn into_component_statements(
        &self,
        parent: &str,
        index: usize,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        if fastn_js::utils::is_kernel(self.name.as_str()) {
            let kernel = fastn_js::Kernel::from_component(self.name.as_str(), parent, index);
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
