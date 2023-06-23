#![allow(dead_code)]

#[cfg(test)]
#[macro_use]
mod ftd_test_helpers;
mod element;
mod utils;
mod value;

pub use element::Element;
pub use value::Value;

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
        if ftd::js::element::is_kernel(self.name.as_str()) {
            ftd::js::Element::from_interpreter_component(self)
                .into_component_statements(parent, index)
        } else {
            todo!()
        }
    }
}
