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

    let mut asts = vec![ftd::js::from_tree(document.tree.as_slice(), &document.data)];
    let default_thing_name = ftd::interpreter::default::default_bag()
        .into_iter()
        .map(|v| v.0)
        .collect_vec();
    for (key, thing) in document.data.iter() {
        if default_thing_name.contains(&key) {
            continue;
        }
        if let ftd::interpreter::Thing::Component(c) = thing {
            asts.push(c.to_ast(&document.data));
        } else if let ftd::interpreter::Thing::Variable(v) = thing {
            asts.push(v.to_ast());
        } else if let ftd::interpreter::Thing::Function(f) = thing {
            asts.push(f.to_ast());
        }
    }
    asts
}

impl ftd::interpreter::Function {
    pub fn to_ast(&self) -> fastn_js::Ast {
        use itertools::Itertools;

        fastn_js::udf_with_params(
            self.name.as_str(),
            self.expression
                .iter()
                .map(|e| {
                    fastn_grammar::evalexpr::build_operator_tree(e.expression.as_str()).unwrap()
                })
                .collect_vec(),
            self.arguments
                .iter()
                .map(|v| v.name.to_string())
                .collect_vec(),
        )
    }
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
    pub fn to_ast(&self, bag: &ftd::Map<ftd::interpreter::Thing>) -> fastn_js::Ast {
        use itertools::Itertools;

        let mut statements = vec![];
        statements.extend(self.definition.to_component_statements("parent", 0, bag));
        fastn_js::component_with_params(
            self.name.as_str(),
            statements,
            self.arguments
                .iter()
                .map(|v| v.name.to_string())
                .collect_vec(),
        )
    }
}

pub fn from_tree(
    tree: &[ftd::interpreter::Component],
    bag: &ftd::Map<ftd::interpreter::Thing>,
) -> fastn_js::Ast {
    let mut statements = vec![];
    for (index, component) in tree.iter().enumerate() {
        statements.extend(component.to_component_statements("parent", index, bag))
    }
    fastn_js::component0("main", statements)
}

impl ftd::interpreter::Component {
    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        bag: &ftd::Map<ftd::interpreter::Thing>,
    ) -> Vec<fastn_js::ComponentStatement> {
        use itertools::Itertools;

        if ftd::js::element::is_kernel(self.name.as_str()) {
            ftd::js::Element::from_interpreter_component(self)
                .to_component_statements(parent, index)
        } else if let Some(ftd::interpreter::Thing::Component(component_definition)) =
            bag.get(self.name.as_str())
        {
            let arguments = component_definition
                .arguments
                .iter()
                .map(|v| {
                    v.get_value(self.properties.as_slice())
                        .to_set_property_value()
                })
                .collect_vec();
            vec![fastn_js::ComponentStatement::InstantiateComponent(
                fastn_js::InstantiateComponent {
                    name: self.name.to_string(),
                    arguments,
                    parent: parent.to_string(),
                },
            )]
        } else {
            panic!("Can't find, {}", self.name)
        }
    }
}
