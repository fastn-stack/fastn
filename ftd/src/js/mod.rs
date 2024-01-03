#![allow(dead_code)]

#[cfg(test)]
#[macro_use]
mod ftd_test_helpers;
mod element;
mod resolver;
mod utils;
mod value;

pub use element::{Common, Element};
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
    use itertools::Itertools;
    let doc = ftd::interpreter::TDoc::new(&document.name, &document.aliases, &document.data);
    // Check if document tree has rive. This is used to add rive script.
    // dbg!(&document.tree);
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
                variants: fastn_js::SetPropertyValue::Value(fastn_js::Value::Record {
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

impl ftd::interpreter::Function {
    pub fn to_ast(&self, doc: &ftd::interpreter::TDoc) -> fastn_js::Ast {
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

impl ftd::interpreter::Variable {
    pub fn to_ast(
        &self,
        doc: &ftd::interpreter::TDoc,
        prefix: Option<String>,
        has_rive_components: &mut bool,
    ) -> fastn_js::Ast {
        if let Ok(value) = self.value.value(doc.name, self.value.line_number()) {
            if let ftd::interpreter::Kind::Record { name } = &self.kind.kind {
                let record = doc.get_record(name, self.line_number).unwrap();
                let record_fields = value
                    .record_fields(doc.name, self.value.line_number())
                    .unwrap();
                let mut fields = vec![];
                for field in record.fields {
                    if let Some(value) = record_fields.get(field.name.as_str()) {
                        fields.push((
                            field.name.to_string(),
                            value.to_fastn_js_value_with_none(doc, has_rive_components),
                        ));
                    } else {
                        fields.push((
                            field.name.to_string(),
                            field
                                .get_default_value()
                                .unwrap()
                                .to_set_property_value_with_none(doc, has_rive_components),
                        ));
                    }
                }
                return fastn_js::Ast::RecordInstance(fastn_js::RecordInstance {
                    name: self.name.to_string(),
                    fields: fastn_js::SetPropertyValue::Value(fastn_js::Value::Record {
                        fields,
                        other_references: vec![],
                    }),
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

impl ftd::interpreter::ComponentDefinition {
    pub fn to_ast(
        &self,
        doc: &ftd::interpreter::TDoc,
        has_rive_components: &mut bool,
    ) -> fastn_js::Ast {
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
    tree: &[ftd::interpreter::Component],
    doc: &ftd::interpreter::TDoc,
    has_rive_components: &mut bool,
) -> fastn_js::Ast {
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

impl ftd::interpreter::Component {
    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        use itertools::Itertools;

        let loop_alias = self.iteration.clone().map(|v| v.alias);
        let loop_counter_alias = self.iteration.clone().and_then(|v| {
            if let Some(ref loop_counter_alias) = v.loop_counter_alias {
                let (_, loop_counter_alias, _remaining) =
                    ftd::interpreter::utils::get_doc_name_and_thing_name_and_remaining(
                        loop_counter_alias.as_str(),
                        doc.name,
                        v.line_number,
                    );
                return Some(loop_counter_alias);
            }
            None
        });
        let mut component_statements = if self.is_loop() || self.condition.is_some() {
            self.to_component_statements_(
                fastn_js::FUNCTION_PARENT,
                0,
                doc,
                &rdata.clone_with_new_loop_alias(
                    &loop_alias,
                    &loop_counter_alias,
                    doc.name.to_string(),
                ),
                true,
                has_rive_components,
            )
        } else {
            self.to_component_statements_(
                parent,
                index,
                doc,
                &rdata.clone_with_new_loop_alias(&None, &None, doc.name.to_string()),
                should_return,
                has_rive_components,
            )
        };

        if let Some(condition) = self.condition.as_ref() {
            component_statements = vec![fastn_js::ComponentStatement::ConditionalComponent(
                fastn_js::ConditionalComponent {
                    deps: condition
                        .references
                        .values()
                        .flat_map(|v| {
                            v.get_deps(&rdata.clone_with_new_loop_alias(
                                &loop_alias,
                                &loop_counter_alias,
                                doc.name.to_string(),
                            ))
                        })
                        .collect_vec(),
                    condition: condition.update_node_with_variable_reference_js(
                        &rdata.clone_with_new_loop_alias(
                            &loop_alias,
                            &loop_counter_alias,
                            doc.name.to_string(),
                        ),
                    ),
                    statements: component_statements,
                    parent: parent.to_string(),
                    should_return: self.is_loop() || should_return,
                },
            )];
        }

        if let Some(iteration) = self.iteration.as_ref() {
            component_statements = vec![fastn_js::ComponentStatement::ForLoop(fastn_js::ForLoop {
                list_variable: iteration.on.to_fastn_js_value(
                    doc,
                    &rdata.clone_with_new_loop_alias(
                        &loop_alias,
                        &loop_counter_alias,
                        doc.name.to_string(),
                    ),
                    false,
                ),
                statements: component_statements,
                parent: parent.to_string(),
                should_return,
            })];
        }

        component_statements
    }

    fn to_component_statements_(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        if let Some(kernel_component_statements) = self.kernel_to_component_statements(
            parent,
            index,
            doc,
            rdata,
            should_return,
            has_rive_components,
        ) {
            kernel_component_statements
        } else if let Some(defined_component_statements) = self
            .defined_component_to_component_statements(
                parent,
                index,
                doc,
                rdata,
                should_return,
                has_rive_components,
            )
        {
            defined_component_statements
        } else if let Some(header_defined_component_statements) = self
            .header_defined_component_to_component_statements(
                parent,
                index,
                doc,
                rdata,
                should_return,
                has_rive_components,
            )
        {
            header_defined_component_statements
        } else if let Some(variable_defined_component_to_component_statements) = self
            .variable_defined_component_to_component_statements(
                parent,
                index,
                doc,
                rdata,
                should_return,
                has_rive_components,
            )
        {
            variable_defined_component_to_component_statements
        } else {
            panic!("Can't find, {}", self.name)
        }
    }

    fn kernel_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>> {
        if ftd::js::element::is_kernel(self.name.as_str()) {
            if !*has_rive_components {
                *has_rive_components = ftd::js::element::is_rive_component(self.name.as_str());
            }
            Some(
                ftd::js::Element::from_interpreter_component(self, doc).to_component_statements(
                    parent,
                    index,
                    doc,
                    rdata,
                    should_return,
                    has_rive_components,
                ),
            )
        } else {
            None
        }
    }

    fn defined_component_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>> {
        if let Some(arguments) =
            ftd::js::utils::get_set_property_values_for_provided_component_properties(
                doc,
                rdata,
                self.name.as_str(),
                self.properties.as_slice(),
                self.line_number,
                has_rive_components,
            )
        {
            let mut component_statements = vec![];
            let instantiate_component = fastn_js::InstantiateComponent::new(
                self.name.as_str(),
                arguments,
                parent,
                rdata.inherited_variable_name,
                should_return,
                index,
                false,
            );

            let instantiate_component_var_name = instantiate_component.var_name.clone();

            component_statements.push(fastn_js::ComponentStatement::InstantiateComponent(
                instantiate_component,
            ));

            component_statements.extend(self.events.iter().filter_map(|event| {
                event
                    .to_event_handler_js(instantiate_component_var_name.as_str(), doc, rdata)
                    .map(|event_handler| {
                        fastn_js::ComponentStatement::AddEventHandler(event_handler)
                    })
            }));

            Some(component_statements)
        } else {
            None
        }
    }

    // ftd.ui type header
    fn header_defined_component_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>> {
        let (component_name, remaining) = ftd::interpreter::utils::get_doc_name_and_remaining(
            self.name.as_str(),
            doc.name,
            self.line_number,
        );

        let remaining = remaining?;

        match rdata.component_definition_name {
            Some(ref component_definition_name) if component_name.eq(component_definition_name) => {
            }
            _ => return None,
        }

        let component = doc
            .get_component(component_name.as_str(), self.line_number)
            .ok()?;

        let mut arguments = vec![];

        if let Some(component_name) =
            ftd::js::utils::is_module_argument(component.arguments.as_slice(), remaining.as_str())
        {
            arguments = ftd::js::utils::get_set_property_values_for_provided_component_properties(
                doc,
                rdata,
                component_name.as_str(),
                self.properties.as_slice(),
                self.line_number,
                has_rive_components,
            )?;
        } else if !ftd::js::utils::is_ui_argument(
            component.arguments.as_slice(),
            remaining.as_str(),
        ) {
            return None;
        }

        let value = ftd::js::Value::Reference(self.name.to_owned()).to_set_property_value_with_ui(
            doc,
            rdata,
            has_rive_components,
            should_return,
        );
        let instantiate_component = fastn_js::InstantiateComponent::new_with_definition(
            value,
            arguments,
            parent,
            rdata.inherited_variable_name,
            should_return,
            index,
            true,
        );

        let mut component_statements = vec![];
        let instantiate_component_var_name = instantiate_component.var_name.clone();

        component_statements.push(fastn_js::ComponentStatement::InstantiateComponent(
            instantiate_component,
        ));

        component_statements.extend(self.events.iter().filter_map(|event| {
            event
                .to_event_handler_js(&instantiate_component_var_name, doc, rdata)
                .map(fastn_js::ComponentStatement::AddEventHandler)
        }));

        Some(component_statements)
    }

    fn variable_defined_component_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>> {
        /*
        Todo: Check if the `self.name` is a loop-alias of `ftd.ui list` variable and then
         uncomment the bellow code which checks for `self.name` as variable of `ftd.ui` type
        if !doc
            .get_variable(self.name.as_str(), self.line_number)
            .ok()?
            .kind
            .is_ui()
        {
            return None;
        }*/

        // The reference `self.name` is either the ftd.ui type variable or the loop-alias
        let value = ftd::js::Value::Reference(self.name.to_owned()).to_set_property_value_with_ui(
            doc,
            rdata,
            has_rive_components,
            should_return,
        );

        let instantiate_component = fastn_js::InstantiateComponent::new_with_definition(
            value,
            vec![],
            parent,
            rdata.inherited_variable_name,
            should_return,
            index,
            true,
        );

        let mut component_statements = vec![];
        let instantiate_component_var_name = instantiate_component.var_name.clone();

        component_statements.push(fastn_js::ComponentStatement::InstantiateComponent(
            instantiate_component,
        ));

        component_statements.extend(self.events.iter().filter_map(|event| {
            event
                .to_event_handler_js(&instantiate_component_var_name, doc, rdata)
                .map(fastn_js::ComponentStatement::AddEventHandler)
        }));

        Some(component_statements)
    }
}

impl ftd::interpreter::WebComponentDefinition {
    pub fn to_ast(&self, doc: &ftd::interpreter::TDoc) -> fastn_js::Ast {
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
