#![allow(dead_code)]

#[cfg(test)]
#[macro_use]
mod ftd_test_helpers;
mod element;
mod utils;
mod value;

pub use element::{Common, Element};
pub use value::Value;

pub fn document_into_js_ast(document: ftd::interpreter::Document) -> Vec<fastn_js::Ast> {
    use itertools::Itertools;

    let mut asts = vec![ftd::js::from_tree(document.tree.as_slice())];
    let default_thing_name = ftd::interpreter::default::default_bag()
        .into_iter()
        .map(|v| v.0)
        .collect_vec();
    for (key, thing) in document.data {
        if default_thing_name.contains(&key) {
            continue;
        }
        if let ftd::interpreter::Thing::Component(c) = thing {
            asts.push(c.to_ast());
        } else if let ftd::interpreter::Thing::Variable(v) = thing {
            asts.push(v.to_ast());
        }
    }
    asts
}

impl ftd::interpreter::Variable {
    pub fn to_ast(&self) -> fastn_js::Ast {
        if self.mutable {
            fastn_js::Ast::MutableVariable(fastn_js::MutableVariable {
                name: self.name.to_string(),
                value: self.value.to_fastn_js_value().to_js(),
                is_quoted: false,
            })
        } else {
            fastn_js::Ast::StaticVariable(fastn_js::StaticVariable {
                name: self.name.to_string(),
                value: self.value.to_fastn_js_value().to_js(),
                is_quoted: false,
            })
        }
    }
}

impl ftd::interpreter::ComponentDefinition {
    pub fn to_ast(&self) -> fastn_js::Ast {
        let mut statements = vec![];
        statements.extend(self.definition.to_component_statements("parent", 0));
        fastn_js::component0(self.name.as_str(), statements)
    }
}

pub fn from_tree(tree: &[ftd::interpreter::Component]) -> fastn_js::Ast {
    let mut statements = vec![];
    for (index, component) in tree.iter().enumerate() {
        statements.extend(component.to_component_statements("parent", index))
    }
    fastn_js::component0("main", statements)
}

impl ftd::interpreter::Component {
    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
    ) -> Vec<fastn_js::ComponentStatement> {
        if ftd::js::element::is_kernel(self.name.as_str()) {
            ftd::js::Element::from_interpreter_component(self)
                .to_component_statements(parent, index)
        } else {
            dbg!(&self.name);
            todo!()
        }
    }
}
