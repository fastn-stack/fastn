#![allow(dead_code)]

#[cfg(test)]
#[macro_use]
mod ftd_test_helpers;
mod element;
pub(crate) mod fastn_type_functions;
mod resolver;
mod utils;
mod value;

pub use element::{Common, Element};
use ftd::js::value::ArgumentExt;
pub use resolver::ResolverData;
pub use value::Value;

pub const CODE_DEFAULT_THEME: &str = "fastn-theme.dark";

pub fn all_js_without_test(package_name: &str) -> String {
    let all_js = fastn_js::all_js_without_test();
    let default_bag_js = fastn_js::to_js(default_bag_into_js_ast().as_slice(), package_name);
    format!("{all_js}\n{default_bag_js}")
}

/// This returns asts of things present in `ftd` module or `default_bag`
pub fn default_bag_into_js_ast() -> Vec<fastn_js::Ast> {
    let mut ftd_asts = vec![];
    let bag = ftd::interpreter::default::get_default_bag();
    let doc = ftd::interpreter::TDoc {
        name: "",
        aliases: &ftd::interpreter::default::default_aliases(),
        bag: ftd::interpreter::BagOrState::Bag(bag),
    };
    let mut export_asts = vec![];
    for thing in ftd::interpreter::default::get_default_bag().values() {
        if let ftd::interpreter::Thing::Variable(v) = thing {
            ftd_asts.push(v.to_ast(&doc, None, &mut false));
        } else if let ftd::interpreter::Thing::Function(f) = thing {
            if f.external_implementation {
                continue;
            }
            ftd_asts.push(f.to_ast(&doc));
        } else if let ftd::interpreter::Thing::Export { from, to, .. } = thing {
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

#[derive(Debug)]
pub struct JSAstData {
    /// This contains asts of things (other than `ftd`) and instructions/tree
    pub asts: Vec<fastn_js::Ast>,
    /// This contains external scripts provided by user and also `ftd`
    /// internally supports (like rive).
    pub scripts: Vec<String>,
}

pub fn document_into_js_ast(document: ftd::interpreter::Document) -> JSAstData {
    use ftd::js::fastn_type_functions::PropertyValueExt;
    use itertools::Itertools;

    let doc = ftd::interpreter::TDoc::new(&document.name, &document.aliases, &document.data);
    // Check if document tree has rive. This is used to add rive script.
    let mut has_rive_components = false;
    let mut document_asts = vec![ftd::js::from_tree(
        document.tree.as_slice(),
        &doc,
        &mut has_rive_components,
    )];
    let default_thing_name = ftd::interpreter::default::get_default_bag()
        .into_iter()
        .map(|v| v.0)
        .collect_vec();

    let mut export_asts = vec![];

    for (key, thing) in document.data.iter() {
        if default_thing_name.contains(&key) {
            continue;
        }
        if let ftd::interpreter::Thing::Component(c) = thing {
            document_asts.push(c.to_ast(&doc, &mut has_rive_components));
        } else if let ftd::interpreter::Thing::Variable(v) = thing {
            document_asts.push(v.to_ast(
                &doc,
                Some(fastn_js::GLOBAL_VARIABLE_MAP.to_string()),
                &mut has_rive_components,
            ));
        } else if let ftd::interpreter::Thing::WebComponent(web_component) = thing {
            document_asts.push(web_component.to_ast(&doc));
        } else if let ftd::interpreter::Thing::Function(f) = thing {
            document_asts.push(f.to_ast(&doc));
        } else if let ftd::interpreter::Thing::Export { from, to, .. } = thing {
            if doc.get_record(from, 0).is_ok() {
                continue;
            }
            export_asts.push(fastn_js::Ast::Export {
                from: from.to_string(),
                to: to.to_string(),
            })
        } else if let ftd::interpreter::Thing::OrType(ot) = thing {
            let mut fields = vec![];
            for variant in &ot.variants {
                if let Some(value) = &variant.clone().fields().get(0).unwrap().value {
                    fields.push((
                        variant
                            .name()
                            .trim_start_matches(
                                format!(
                                    "{}.",
                                    ftd::interpreter::OrType::or_type_name(ot.name.as_str())
                                )
                                .as_str(),
                            )
                            .to_string(),
                        value.to_fastn_js_value_with_none(&doc, &mut false),
                    ));
                }
            }
            document_asts.push(fastn_js::Ast::OrType(fastn_js::OrType {
                name: ot.name.clone(),
                variant: fastn_js::SetPropertyValue::Value(fastn_js::Value::Record {
                    fields,
                    other_references: vec![],
                }),
                prefix: Some(fastn_js::GLOBAL_VARIABLE_MAP.to_string()),
            }));
        }
    }

    document_asts.extend(export_asts);
    let mut scripts = ftd::js::utils::get_external_scripts(has_rive_components);
    scripts.push(ftd::js::utils::get_js_html(
        document.js.into_iter().collect_vec().as_slice(),
    ));
    scripts.push(ftd::js::utils::get_css_html(
        document.css.into_iter().collect_vec().as_slice(),
    ));

    JSAstData {
        asts: document_asts,
        scripts,
    }
}

pub(crate) trait FunctionExt {
    fn to_ast(&self, doc: &ftd::interpreter::TDoc) -> fastn_js::Ast;
}
impl FunctionExt for fastn_type::Function {
    fn to_ast(&self, doc: &ftd::interpreter::TDoc) -> fastn_js::Ast {
        use itertools::Itertools;

        fastn_js::udf_with_arguments(
            self.name.as_str(),
            self.expression
                .iter()
                .map(|e| {
                    fastn_grammar::evalexpr::build_operator_tree(e.expression.as_str()).unwrap()
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
                                    &ftd::js::ResolverData::new_with_component_definition_name(
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

pub(crate) trait VariableExt {
    fn to_ast(
        &self,
        doc: &ftd::interpreter::TDoc,
        prefix: Option<String>,
        has_rive_components: &mut bool,
    ) -> fastn_js::Ast;
}

impl VariableExt for fastn_type::Variable {
    fn to_ast(
        &self,
        doc: &ftd::interpreter::TDoc,
        prefix: Option<String>,
        has_rive_components: &mut bool,
    ) -> fastn_js::Ast {
        use ftd::interpreter::PropertyValueExt;
        use ftd::js::fastn_type_functions::{PropertyValueExt as _, ValueExt as _};

        if let Ok(value) = self.value.value(doc.name, self.value.line_number()) {
            if self.kind.is_record() {
                return fastn_js::Ast::RecordInstance(fastn_js::RecordInstance {
                    name: self.name.to_string(),
                    fields: value.to_fastn_js_value(
                        doc,
                        &ftd::js::ResolverData::none(),
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

pub(crate) trait ComponentDefinitionExt {
    fn to_ast(&self, doc: &ftd::interpreter::TDoc, has_rive_components: &mut bool)
        -> fastn_js::Ast;
}
impl ComponentDefinitionExt for fastn_type::ComponentDefinition {
    fn to_ast(
        &self,
        doc: &ftd::interpreter::TDoc,
        has_rive_components: &mut bool,
    ) -> fastn_js::Ast {
        use ftd::js::fastn_type_functions::ComponentExt;
        use itertools::Itertools;

        let mut statements = vec![];
        statements.extend(self.definition.to_component_statements(
            fastn_js::COMPONENT_PARENT,
            0,
            doc,
            &ftd::js::ResolverData::new_with_component_definition_name(&Some(
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
                                &ftd::js::ResolverData::new_with_component_definition_name(&Some(
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
    tree: &[fastn_type::Component],
    doc: &ftd::interpreter::TDoc,
    has_rive_components: &mut bool,
) -> fastn_js::Ast {
    use ftd::js::fastn_type_functions::ComponentExt;

    let mut statements = vec![];
    for (index, component) in tree.iter().enumerate() {
        statements.extend(component.to_component_statements(
            fastn_js::COMPONENT_PARENT,
            index,
            doc,
            &ftd::js::ResolverData::none(),
            false,
            has_rive_components,
        ))
    }
    fastn_js::component0(fastn_js::MAIN_FUNCTION, statements)
}


pub trait WebComponentDefinitionExt {
    fn to_ast(&self, doc: &ftd::interpreter::TDoc) -> fastn_js::Ast;
}

impl WebComponentDefinitionExt for  fastn_type::WebComponentDefinition {
    fn to_ast(&self, doc: &ftd::interpreter::TDoc) -> fastn_js::Ast {
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
                                &ftd::js::ResolverData::new_with_component_definition_name(&Some(
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
