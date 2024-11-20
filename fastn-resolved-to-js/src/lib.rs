extern crate self as fastn_resolved_to_js;

mod resolver;
pub use resolver::ResolverData;

mod value;
use value::ArgumentExt;
pub use value::Value;

mod element;
use element::Element;

mod fastn_type_functions;
pub use fastn_type_functions::{ComponentExt, PropertyValueExt};
pub mod utils;

pub const CODE_DEFAULT_THEME: &str = "fastn-theme.dark";
pub const REFERENCE: &str = "$";
pub const CLONE: &str = "*$";

pub trait FunctionExt {
    fn to_ast(&self, doc: &dyn fastn_resolved::tdoc::TDoc) -> fastn_js::Ast;
}
impl FunctionExt for fastn_resolved::Function {
    fn to_ast(&self, doc: &dyn fastn_resolved::tdoc::TDoc) -> fastn_js::Ast {
        use itertools::Itertools;

        fastn_js::udf_with_arguments(
            self.name.as_str(),
            self.expression
                .iter()
                .map(|e| {
                    fastn_resolved::evalexpr::build_operator_tree(e.expression.as_str()).unwrap()
                })
                .collect_vec(),
            self.arguments
                .iter()
                .map(|v| {
                    v.get_default_value()
                        .map(|val| {
                            (
                                v.name.to_string(),
                                val.to_set_property_value(
                                    doc,
                                    &fastn_resolved_to_js::ResolverData::new_with_component_definition_name(
                                        &Some(self.name.to_string()),
                                    ),
                                ),
                            )
                        })
                        .unwrap_or_else(|| {
                            (v.name.to_string(), fastn_js::SetPropertyValue::undefined())
                        })
                })
                .collect_vec(),
            self.js.is_some(),
        )
    }
}

pub trait VariableExt {
    fn to_ast(
        &self,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        prefix: Option<String>,
        has_rive_components: &mut bool,
    ) -> fastn_js::Ast;
}

impl VariableExt for fastn_resolved::Variable {
    fn to_ast(
        &self,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        prefix: Option<String>,
        has_rive_components: &mut bool,
    ) -> fastn_js::Ast {
        use fastn_resolved_to_js::fastn_type_functions::{PropertyValueExt, ValueExt};

        if let Some(value) = self.value.value_optional() {
            if self.kind.is_record() {
                return fastn_js::Ast::RecordInstance(fastn_js::RecordInstance {
                    name: self.name.to_string(),
                    fields: value.to_fastn_js_value(
                        doc,
                        &fastn_resolved_to_js::ResolverData::none(),
                        has_rive_components,
                        false,
                    ),
                    prefix,
                });
            } else if self.kind.is_list() {
                // Todo: It should be only for Mutable not Static
                return fastn_js::Ast::MutableList(fastn_js::MutableList {
                    name: self.name.to_string(),
                    value: self
                        .value
                        .to_fastn_js_value_with_none(doc, has_rive_components),
                    prefix,
                });
            } else if self.mutable {
                return fastn_js::Ast::MutableVariable(fastn_js::MutableVariable {
                    name: self.name.to_string(),
                    value: self
                        .value
                        .to_fastn_js_value_with_none(doc, has_rive_components),
                    prefix,
                });
            }
        }
        fastn_js::Ast::StaticVariable(fastn_js::StaticVariable {
            name: self.name.to_string(),
            value: self
                .value
                .to_fastn_js_value_with_none(doc, has_rive_components),
            prefix,
        })
    }
}

pub trait ComponentDefinitionExt {
    fn to_ast(
        &self,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        has_rive_components: &mut bool,
    ) -> fastn_js::Ast;
}
impl ComponentDefinitionExt for fastn_resolved::ComponentDefinition {
    fn to_ast(
        &self,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        has_rive_components: &mut bool,
    ) -> fastn_js::Ast {
        use fastn_resolved_to_js::fastn_type_functions::ComponentExt;
        use itertools::Itertools;

        let mut statements = vec![];
        statements.extend(self.definition.to_component_statements(
            fastn_js::COMPONENT_PARENT,
            0,
            doc,
            &fastn_resolved_to_js::ResolverData::new_with_component_definition_name(&Some(
                self.name.to_string(),
            )),
            true,
            has_rive_components,
        ));
        fastn_js::component_with_params(
            self.name.as_str(),
            statements,
            self.arguments
                .iter()
                .flat_map(|v| {
                    v.get_default_value().map(|val| {
                        (
                            v.name.to_string(),
                            val.to_set_property_value_with_ui(
                                doc,
                                &fastn_resolved_to_js::ResolverData::new_with_component_definition_name(&Some(
                                    self.name.to_string(),
                                )),
                                has_rive_components,
                                false,
                            ),
                            v.mutable.to_owned(),
                        )
                    })
                })
                .collect_vec(),
        )
    }
}

pub fn from_tree(
    tree: &[fastn_resolved::ComponentInvocation],
    doc: &dyn fastn_resolved::tdoc::TDoc,
    has_rive_components: &mut bool,
) -> fastn_js::Ast {
    use fastn_resolved_to_js::fastn_type_functions::ComponentExt;

    let mut statements = vec![];
    for (index, component) in tree.iter().enumerate() {
        statements.extend(component.to_component_statements(
            fastn_js::COMPONENT_PARENT,
            index,
            doc,
            &fastn_resolved_to_js::ResolverData::none(),
            false,
            has_rive_components,
        ))
    }
    fastn_js::component0(fastn_js::MAIN_FUNCTION, statements)
}

pub trait WebComponentDefinitionExt {
    fn to_ast(&self, doc: &dyn fastn_resolved::tdoc::TDoc) -> fastn_js::Ast;
}

impl WebComponentDefinitionExt for fastn_resolved::WebComponentDefinition {
    fn to_ast(&self, doc: &dyn fastn_resolved::tdoc::TDoc) -> fastn_js::Ast {
        use itertools::Itertools;

        let kernel = fastn_js::Kernel::from_component(
            fastn_js::ElementKind::WebComponent(self.name.clone()),
            fastn_js::COMPONENT_PARENT,
            0,
        );

        let statements = vec![
            fastn_js::ComponentStatement::CreateKernel(kernel.clone()),
            fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            },
        ];

        fastn_js::component_with_params(
            self.name.as_str(),
            statements,
            self.arguments
                .iter()
                .flat_map(|v| {
                    v.get_default_value().map(|val| {
                        (
                            v.name.to_string(),
                            val.to_set_property_value(
                                doc,
                                &fastn_resolved_to_js::ResolverData::new_with_component_definition_name(&Some(
                                    self.name.to_string(),
                                )),
                            ),
                            v.mutable.to_owned(),
                        )
                    })
                })
                .collect_vec(),
        )
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct VecMap<T> {
    value: fastn_builtins::Map<Vec<T>>,
}

impl<T: std::cmp::PartialEq> VecMap<T> {
    pub fn new() -> VecMap<T> {
        VecMap {
            value: Default::default(),
        }
    }

    pub fn insert(&mut self, key: String, value: T) {
        if let Some(v) = self.value.get_mut(&key) {
            v.push(value);
        } else {
            self.value.insert(key, vec![value]);
        }
    }

    pub fn unique_insert(&mut self, key: String, value: T) {
        if let Some(v) = self.value.get_mut(&key) {
            if !v.contains(&value) {
                v.push(value);
            }
        } else {
            self.value.insert(key, vec![value]);
        }
    }

    pub fn extend(&mut self, key: String, value: Vec<T>) {
        if let Some(v) = self.value.get_mut(&key) {
            v.extend(value);
        } else {
            self.value.insert(key, value);
        }
    }

    pub fn get_value(&self, key: &str) -> Vec<&T> {
        self.get_value_and_rem(key)
            .into_iter()
            .map(|(k, _)| k)
            .collect()
    }

    pub fn get_value_and_rem(&self, key: &str) -> Vec<(&T, Option<String>)> {
        let mut values = vec![];

        self.value.iter().for_each(|(k, v)| {
            if k.eq(key) {
                values.extend(
                    v.iter()
                        .map(|a| (a, None))
                        .collect::<Vec<(&T, Option<String>)>>(),
                );
            } else if let Some(rem) = key.strip_prefix(format!("{}.", k).as_str()) {
                values.extend(
                    v.iter()
                        .map(|a| (a, Some(rem.to_string())))
                        .collect::<Vec<(&T, Option<String>)>>(),
                );
            } else if let Some(rem) = k.strip_prefix(format!("{}.", key).as_str()) {
                values.extend(
                    v.iter()
                        .map(|a| (a, Some(rem.to_string())))
                        .collect::<Vec<(&T, Option<String>)>>(),
                );
            }
        });
        values
    }
}

pub fn default_bag_into_js_ast(doc: &dyn fastn_resolved::tdoc::TDoc) -> Vec<fastn_js::Ast> {
    let mut ftd_asts = vec![];

    let mut export_asts = vec![];
    for thing in fastn_builtins::get_default_bag().values() {
        if let fastn_resolved::Definition::Variable(v) = thing {
            ftd_asts.push(v.to_ast(doc, None, &mut false));
        } else if let fastn_resolved::Definition::Function(f) = thing {
            if f.external_implementation {
                continue;
            }
            ftd_asts.push(f.to_ast(doc));
        } else if let fastn_resolved::Definition::Export { from, to, .. } = thing {
            export_asts.push(fastn_js::Ast::Export {
                from: from.to_string(),
                to: to.to_string(),
            })
        }
    }

    // Global default inherited variable
    ftd_asts.push(fastn_js::Ast::StaticVariable(fastn_js::StaticVariable {
        name: "inherited".to_string(),
        value: fastn_js::SetPropertyValue::Value(fastn_js::Value::Record {
            fields: vec![
                (
                    "colors".to_string(),
                    fastn_js::SetPropertyValue::Reference(
                        "ftd#default-colors__DOT__getClone()__DOT__setAndReturn\
                        (\"is_root\"__COMMA__\
                         true)"
                            .to_string(),
                    ),
                ),
                (
                    "types".to_string(),
                    fastn_js::SetPropertyValue::Reference(
                        "ftd#default-types__DOT__getClone()__DOT__setAndReturn\
                        (\"is_root\"__COMMA__\
                         true)"
                            .to_string(),
                    ),
                ),
            ],
            other_references: vec![],
        }),
        prefix: None,
    }));

    ftd_asts.extend(export_asts);
    ftd_asts
}
