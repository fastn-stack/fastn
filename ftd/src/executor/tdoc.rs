#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: &'a mut ftd::Map<ftd::interpreter2::Thing>,
    pub dummy_instructions: &'a mut ftd::VecMap<ftd::executor::DummyElement>,
    pub element_constructor: &'a mut ftd::Map<ftd::executor::ElementConstructor>,
    pub js: &'a mut std::collections::HashSet<String>,
    pub css: &'a mut std::collections::HashSet<String>,
}

impl<'a> TDoc<'a> {
    pub(crate) fn itdoc(&self) -> ftd::interpreter2::TDoc {
        ftd::interpreter2::TDoc::new(self.name, self.aliases, self.bag)
    }

    pub fn resolve_all_self_references(
        name: String,
        component_name: &str,
        map: &ftd::Map<(String, Option<String>)>,
    ) -> String {
        let mut resolved_value = name;
        loop {
            if resolved_value.starts_with(format!("{}.", component_name).as_str()) {
                resolved_value = map.get(resolved_value.as_str()).cloned().unwrap().0;
            } else {
                break;
            }
        }
        resolved_value
    }

    pub(crate) fn resolve_self_referenced_values(
        &mut self,
        component_name: &str,
        map: &ftd::Map<(String, Option<String>)>,
    ) -> ftd::executor::Result<ftd::Map<String>> {
        let mut resolved_map: ftd::Map<String> = Default::default();

        for (k, (name, has_self_reference)) in map.iter() {
            let name = TDoc::resolve_all_self_references(name.clone(), component_name, map);

            if let Some(has_self_reference) = has_self_reference {
                let variable = match self.bag.get_mut(name.as_str()).unwrap() {
                    ftd::interpreter2::Thing::Variable(v) => v,
                    _ => unreachable!("Reference {} is not a valid variable", name.as_str()),
                };
                let value = TDoc::resolve_all_self_references(
                    has_self_reference.clone(),
                    component_name,
                    map,
                );
                variable.value.set_reference_or_clone(value.as_str());
            }

            resolved_map.insert(k.to_string(), name);
        }
        Ok(resolved_map)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn insert_local_variables(
        &mut self,
        component_name: &str,
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        container: &[usize],
        line_number: usize,
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
        insert_null: bool,
    ) -> ftd::executor::Result<ftd::Map<String>> {
        let mut map: ftd::Map<(String, Option<String>)> = Default::default();
        for argument in arguments {
            if let Some((k, v, has_self_reference)) = self.insert_local_variable(
                component_name,
                properties,
                argument,
                container,
                line_number,
                inherited_variables,
                insert_null,
            )? {
                map.insert(k, (v, has_self_reference));
            }
        }

        let resolved_map = self.resolve_self_referenced_values(component_name, &map)?;
        Ok(resolved_map)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn insert_local_variable(
        &mut self,
        component_name: &str,
        properties: &[ftd::interpreter2::Property],
        argument: &ftd::interpreter2::Argument,
        container: &[usize],
        line_number: usize,
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
        insert_null: bool,
    ) -> ftd::executor::Result<Option<(String, String, Option<String>)>> {
        let string_container = ftd::executor::utils::get_string_container(container);
        let source = argument.to_sources();
        let properties = ftd::interpreter2::utils::find_properties_by_source(
            source.as_slice(),
            properties,
            self.name,
            argument,
            line_number,
        )?;

        let name_in_component_definition = format!("{}.{}", component_name, argument.name);
        if argument.kind.is_module() {
            if let ftd::interpreter2::Value::Module { name, .. } = properties
                .first()
                .unwrap()
                .resolve(&self.itdoc(), &Default::default())?
                // TODO: Remove unwrap()
                .unwrap()
            {
                return Ok(Some((name_in_component_definition, name, None)));
            }
        }

        let (default, conditions) = properties.into_iter().fold(
            (None, vec![]),
            |(mut default, mut conditions), property| {
                if let Some(condition) = property.condition {
                    conditions.push(ftd::interpreter2::ConditionalValue::new(
                        condition,
                        property.value,
                        property.line_number,
                    ));
                } else {
                    default = Some((property.value, property.source.is_default()));
                }
                (default, conditions)
            },
        );

        let (default, is_default_source, is_default_null) = if let Some(default) = default {
            (default.0, default.1, false)
        } else {
            (
                ftd::interpreter2::PropertyValue::Value {
                    value: ftd::interpreter2::Value::Optional {
                        data: Box::new(None),
                        kind: argument.kind.to_owned(),
                    },
                    is_mutable: argument.mutable,
                    line_number,
                },
                false,
                true,
            )
        };

        if is_default_null && !insert_null && conditions.is_empty() {
            return Ok(None);
        }

        let self_reference = {
            let mut self_reference = None;
            if let Some(name) = default.get_reference_or_clone().cloned() {
                if name.starts_with(format!("{}.", component_name).as_str()) {
                    self_reference = Some(name);
                }
            }
            self_reference
        };

        match default.reference_name() {
            Some(name) if conditions.is_empty() => {
                if !is_default_source
                    || !ftd::executor::utils::found_parent_containers(
                        inherited_variables
                            .get_value(argument.name.as_str())
                            .as_slice(),
                        container,
                    )
                {
                    inherited_variables.insert(
                        argument.name.to_string(),
                        (name.to_string(), container.to_vec()),
                    );
                }

                return Ok(Some((name_in_component_definition, name.to_string(), None)));
            }
            _ => {}
        }

        let variable_name = self.itdoc().resolve_name(
            format!("{}:{}:{}", component_name, argument.name, string_container).as_str(),
        );

        if (!is_default_source
            || !ftd::executor::utils::found_parent_containers(
                inherited_variables
                    .get_value(argument.name.as_str())
                    .as_slice(),
                container,
            ))
            && conditions.is_empty()
        {
            inherited_variables.insert(
                argument.name.to_string(),
                (variable_name.to_string(), container.to_vec()),
            );
        }

        let variable = ftd::interpreter2::Variable {
            name: variable_name.to_string(),
            kind: argument.kind.to_owned(),
            mutable: argument.mutable,
            value: default,
            conditional_value: conditions,
            line_number,
            is_static: true,
        }
        .set_static(&self.itdoc());

        ftd::interpreter2::utils::validate_variable(&variable, &self.itdoc())?;

        self.bag.insert(
            variable.name.to_string(),
            ftd::interpreter2::Thing::Variable(variable),
        );

        Ok(Some((
            name_in_component_definition,
            variable_name,
            self_reference,
        )))
    }
}
