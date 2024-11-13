#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ComponentDefinition {
    pub name: String,
    pub arguments: Vec<Argument>,
    pub definition: fastn_type::Component,
    pub css: Option<fastn_type::PropertyValue>,
    pub line_number: usize,
}

impl ComponentDefinition {
    pub(crate) fn new(
        name: &str,
        arguments: Vec<Argument>,
        definition: Component,
        css: Option<fastn_type::PropertyValue>,
        line_number: usize,
    ) -> ComponentDefinition {
        ComponentDefinition {
            name: name.to_string(),
            arguments,
            definition,
            css,
            line_number,
        }
    }

    pub(crate) fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        use itertools::Itertools;

        let component_definition = ast.get_component_definition(doc.name)?;
        let arguments = component_definition
            .arguments
            .iter()
            .map(|v| v.name.to_string())
            .collect_vec();

        let definition_name_with_arguments =
            (component_definition.name.as_str(), arguments.as_slice());

        Component::scan_ast_component(
            component_definition.definition,
            Some(definition_name_with_arguments),
            doc,
        )?;

        Argument::scan_ast_fields(component_definition.arguments, doc, &Default::default())?;

        Ok(())
    }

    pub(crate) fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<ComponentDefinition>> {
        use ftd::interpreter::PropertyValueExt;

        let component_definition = ast.get_component_definition(doc.name)?;
        let name = doc.resolve_name(component_definition.name.as_str());

        let css = if let Some(ref css) = component_definition.css {
            Some(try_ok_state!(fastn_type::PropertyValue::from_ast_value(
                ftd_ast::VariableValue::String {
                    value: css.to_string(),
                    line_number: component_definition.line_number(),
                    source: ftd_ast::ValueSource::Default,
                    condition: None
                },
                doc,
                false,
                Some(&fastn_type::Kind::string().into_kind_data()),
            )?))
        } else {
            None
        };

        let mut arguments = try_ok_state!(Argument::from_ast_fields(
            component_definition.name.as_str(),
            component_definition.arguments,
            doc,
            &Default::default(),
        )?);

        let definition_name_with_arguments =
            (component_definition.name.as_str(), arguments.as_mut_slice());
        let definition = try_ok_state!(Component::from_ast_component(
            component_definition.definition,
            &mut Some(definition_name_with_arguments),
            doc,
        )?);
        if let Some(iteration) = definition.iteration.as_ref() {
            return Err(ftd::interpreter::Error::ParseError {
                message: "The component definition cannot have loop. Help: use container component as it's parent"
                    .to_string(),
                doc_id: doc.name.to_string(),
                line_number: iteration.line_number,
            });
        }
        Ok(ftd::interpreter::StateWithThing::new_thing(
            ComponentDefinition::new(
                name.as_str(),
                arguments,
                definition,
                css,
                component_definition.line_number,
            ),
        ))
    }

    pub fn to_value(&self, kind: &fastn_type::KindData) -> fastn_type::Value {
        fastn_type::Value::UI {
            name: self.name.to_string(),
            kind: kind.to_owned(),
            component: self.definition.to_owned(),
        }
    }
}

pub type Argument = ftd::interpreter::Field;

#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub enum ComponentSource {
    #[default]
    Declaration,
    Variable,
}

pub(crate) fn get_extra_argument_property_value(
    property: ftd_ast::Property,
    doc_id: String,
) -> ftd::interpreter::Result<Option<(String, fastn_type::PropertyValue)>> {
    if let ftd_ast::PropertySource::Header { name, .. } = property.source.clone() {
        let line_number = property.value.line_number();
        let value = match property.value {
            ftd_ast::VariableValue::String { value, .. } => value,
            value => {
                return Err(ftd::interpreter::Error::InvalidKind {
                    doc_id,
                    line_number: value.line_number(),
                    message: "kw-args currently support only string values.".to_string(),
                })
            }
        };

        return Ok(Some((
            name,
            fastn_type::PropertyValue::Value {
                value: fastn_type::Value::new_string(&value),
                is_mutable: false,
                line_number,
            },
        )));
    }

    Ok(None)
}

pub(crate) fn check_if_property_is_provided_for_required_argument(
    component_arguments: &[ftd::interpreter::Field],
    properties: &[fastn_type::Property],
    component_name: &str,
    line_number: usize,
    doc_id: &str,
) -> ftd::interpreter::Result<()> {
    for argument in component_arguments {
        if !argument.is_value_required() || argument.kind.is_kwargs() {
            continue;
        }
        if argument
            .get_default_interpreter_property_value(properties)
            .map(|v| v.is_none())
            .unwrap_or(true)
        {
            return Err(ftd::interpreter::Error::ParseError {
                message: format!(
                    "Property `{}` of component `{}` is not passed",
                    argument.name, component_name
                ),
                doc_id: doc_id.to_string(),
                line_number,
            });
        }
    }
    Ok(())
}

pub(crate) fn search_things_for_module(
    component_name: &str,
    properties: &[fastn_type::Property],
    doc: &mut ftd::interpreter::TDoc,
    arguments: &[ftd::interpreter::Argument],
    definition_name_with_arguments: &mut Option<(&str, &mut [Argument])>,
    line_number: usize,
) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<()>> {
    use ftd::interpreter::PropertySourceExt;

    for argument in arguments.iter() {
        if !argument.kind.is_module() {
            continue;
        }
        let sources = argument.to_sources();
        let property = ftd::interpreter::utils::find_properties_by_source(
            sources.as_slice(),
            properties,
            doc.name,
            argument,
            argument.line_number,
        )?;
        if property.len() != 1 {
            return ftd::interpreter::utils::e2(
                format!(
                    "Expected one value for `module` type argument `{}`, found `{}` values",
                    argument.name,
                    property.len()
                ),
                doc.name,
                line_number,
            );
        }
        let module_property = property.first().unwrap();
        // TODO: Remove unwrap()

        let (m_name, things) = get_module_name_and_thing(
            module_property,
            doc,
            definition_name_with_arguments,
            argument,
        )?;

        let mut m_alias;
        {
            let current_parsed_document = if let Some(state) = {
                match &mut doc.bag {
                    ftd::interpreter::tdoc::BagOrState::Bag(_) => None,
                    ftd::interpreter::tdoc::BagOrState::State(s) => Some(s),
                }
            } {
                state.parsed_libs.get_mut(state.id.as_str()).unwrap()
            } else {
                return doc.err("not found", m_name, "search_thing", line_number);
            };
            let (module, alias) = ftd_ast::utils::get_import_alias(m_name.as_str());
            if !current_parsed_document
                .doc_aliases
                .contains_key(alias.as_str())
            {
                current_parsed_document
                    .doc_aliases
                    .insert(alias.to_string(), module.to_string());
            }
            m_alias = alias;
        }

        if let Some(m) = doc.aliases.get(m_alias.as_str()) {
            m_alias = m.to_string();
        }

        let mut unresolved_thing = None;

        for (thing, _expected_kind) in things {
            let mut new_doc_name = doc.name.to_string();
            let mut new_doc_aliases = doc.aliases.clone();

            // If the module name (value) is coming from the argument of the component then we
            // need to change doc to the new-doc, else if it's coming from property then no need
            // to change the doc.
            if module_property.source.is_default() {
                // This is needed because the component can be exported from some other module
                // so, we need to fetch this module name in module_name
                // -- import: foo
                // export: bar
                //
                // So the bar component is actually present in foo module and we need foo as
                // value of module_name.
                let component_name = doc
                    .get_thing(component_name, line_number)
                    .map(|v| v.name())
                    .unwrap_or(component_name.to_string());
                let module_name =
                    ftd::interpreter::utils::get_doc_name(component_name.as_str(), doc.name);

                if let Some(state) = doc.state() {
                    let parsed_document = state.parsed_libs.get(module_name.as_str()).unwrap();
                    new_doc_name = parsed_document.name.to_string();
                    new_doc_aliases = parsed_document.doc_aliases.clone();
                }
            }

            let mut new_doc = match &mut doc.bag {
                ftd::interpreter::BagOrState::Bag(bag) => {
                    ftd::interpreter::TDoc::new(&new_doc_name, &new_doc_aliases, bag)
                }
                ftd::interpreter::BagOrState::State(state) => {
                    ftd::interpreter::TDoc::new_state(&new_doc_name, &new_doc_aliases, state)
                }
            };

            let mut m_alias = m_alias.clone();
            if let Some(m) = new_doc.aliases.get(m_alias.as_str()) {
                m_alias = m.to_string();
            }

            let thing_real_name = format!("{}#{}", m_alias, thing);

            if unresolved_thing.is_some() {
                new_doc.scan_thing(&thing_real_name, line_number)?;
            } else {
                let result = new_doc.search_thing(&thing_real_name, line_number)?;
                if !result.is_thing() {
                    unresolved_thing = Some(result);
                } else {
                    //Todo: check with kind, if kind matches with expected_kind
                    try_ok_state!(result);
                }
            }
        }

        if let Some(unresolved_thing) = unresolved_thing {
            try_ok_state!(unresolved_thing);
        }
    }
    Ok(ftd::interpreter::StateWithThing::new_thing(()))
}

fn get_module_name_and_thing(
    module_property: &fastn_type::Property,
    doc: &mut ftd::interpreter::TDoc,
    definition_name_with_arguments: &mut Option<(&str, &mut [Argument])>,
    component_argument: &ftd::interpreter::Argument,
) -> ftd::interpreter::Result<(String, ftd::Map<ftd::interpreter::ModuleThing>)> {
    use ftd::interpreter::{PropertyExt, PropertyValueExt, ValueExt};

    let default_things = {
        let value = if let Some(ref value) = component_argument.value {
            value.clone().resolve(doc, module_property.line_number)?
        } else {
            return ftd::interpreter::utils::e2(
                "Cannot find component argument value for module",
                doc.name,
                component_argument.line_number,
            );
        };

        if let Some(thing) = value.module_thing_optional() {
            thing.clone()
        } else {
            return ftd::interpreter::utils::e2(
                "Cannot find component argument value for module",
                doc.name,
                component_argument.line_number,
            );
        }
    };
    if let Some(module_name) = module_property.value.get_reference_or_clone() {
        if let Some((argument, ..)) =
            ftd::interpreter::utils::get_component_argument_for_reference_and_remaining(
                module_name,
                doc.name,
                definition_name_with_arguments,
                module_property.line_number,
            )?
        {
            if let Some(ref mut property_value) = argument.value {
                if let fastn_type::PropertyValue::Value { value, .. } = property_value {
                    if let Some((name, thing)) = value.mut_module_optional() {
                        thing.extend(default_things);
                        return Ok((name.to_string(), thing.clone()));
                    } else {
                        return ftd::interpreter::utils::e2(
                            format!("Expected module, found: {:?}", property_value),
                            doc.name,
                            module_property.line_number,
                        );
                    }
                }
                match property_value
                    .clone()
                    .resolve(doc, module_property.line_number)?
                {
                    fastn_type::Value::Module { name, things } => return Ok((name, things)),
                    t => {
                        return ftd::interpreter::utils::e2(
                            format!("Expected module, found: {:?}", t),
                            doc.name,
                            module_property.line_number,
                        )
                    }
                }
            }
        }
    }

    match module_property
        .resolve(doc, &Default::default())?
        // TODO: Remove unwrap()
        .unwrap()
    {
        fastn_type::Value::Module { name, things } => Ok((name, things)),
        t => ftd::interpreter::utils::e2(
            format!("Expected module, found: {:?}", t),
            doc.name,
            module_property.line_number,
        ),
    }
}
