#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: &'a mut indexmap::IndexMap<String, ftd::interpreter::Thing>,
    pub dummy_instructions: &'a mut ftd::VecMap<ftd::executor::DummyElement>,
    pub element_constructor: &'a mut ftd::Map<ftd::executor::ElementConstructor>,
    pub js: &'a mut std::collections::HashSet<String>,
    pub css: &'a mut std::collections::HashSet<String>,
    pub rive_data: &'a mut Vec<ftd::executor::RiveData>,
}

impl TDoc<'_> {
    pub(crate) fn itdoc(&self) -> ftd::interpreter::TDoc {
        ftd::interpreter::TDoc::new(self.name, self.aliases, self.bag)
    }

    pub fn resolve_all_self_references(
        name: String,
        component_name: &str,
        map: &ftd::Map<(String, Vec<String>)>,
    ) -> String {
        let mut resolved_value = name;
        loop {
            if resolved_value.starts_with(format!("{}.", component_name).as_str()) {
                let (name, remaining) = {
                    let mut name = resolved_value
                        .strip_prefix(format!("{}.", component_name).as_str())
                        .unwrap()
                        .to_string();
                    let mut remaining = None;
                    if let Some((var_name, var_remaining)) = name.as_str().split_once('.') {
                        remaining = Some(var_remaining.to_string());
                        name = var_name.to_string();
                    }
                    (format!("{}.{}", component_name, name), remaining)
                };

                resolved_value = format!(
                    "{}{}",
                    map.get(name.as_str()).cloned().unwrap().0,
                    if let Some(rem) = remaining {
                        format!(".{}", rem)
                    } else {
                        Default::default()
                    }
                );
            } else {
                break;
            }
        }
        resolved_value
    }

    pub(crate) fn resolve_self_referenced_values(
        &mut self,
        component_name: &str,
        map: &ftd::Map<(String, Vec<String>)>,
    ) -> ftd::executor::Result<ftd::Map<String>> {
        let mut resolved_map: ftd::Map<String> = Default::default();

        for (k, (name, has_self_reference)) in map.iter() {
            let name = TDoc::resolve_all_self_references(name.clone(), component_name, map);

            if !has_self_reference.is_empty() {
                let values: ftd::Map<String> = has_self_reference
                    .iter()
                    .map(|v| {
                        (
                            v.to_string(),
                            TDoc::resolve_all_self_references(v.to_string(), component_name, map),
                        )
                    })
                    .collect();
                let variable = match self.bag.get_mut(name.as_str()).unwrap() {
                    ftd::interpreter::Thing::Variable(v) => v,
                    _ => unreachable!("Reference {} is not a valid variable", name.as_str()),
                };

                set_reference_name(&mut variable.value, &values);
            }

            resolved_map.insert(k.to_string(), name);
        }
        Ok(resolved_map)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn insert_local_variables(
        &mut self,
        component_name: &str,
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        container: &[usize],
        line_number: usize,
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
        insert_null: bool,
    ) -> ftd::executor::Result<ftd::Map<String>> {
        let mut map: ftd::Map<(String, Vec<String>)> = Default::default();
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
        properties: &[ftd::interpreter::Property],
        argument: &ftd::interpreter::Argument,
        container: &[usize],
        line_number: usize,
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
        insert_null: bool,
    ) -> ftd::executor::Result<Option<(String, String, Vec<String>)>> {
        let string_container = ftd::executor::utils::get_string_container(container);
        let source = argument.to_sources();
        let properties = ftd::interpreter::utils::find_properties_by_source(
            source.as_slice(),
            properties,
            self.name,
            argument,
            line_number,
        )?;

        let name_in_component_definition = format!("{}.{}", component_name, argument.name);
        if argument.kind.is_module() {
            if let fastn_type::Value::Module { name, .. } = properties
                .first()
                .unwrap()
                .resolve(&self.itdoc(), &Default::default())?
                // TODO: Remove unwrap()
                .unwrap()
            {
                return Ok(Some((name_in_component_definition, name, vec![])));
            }
        }

        let (default, conditions) = properties.into_iter().fold(
            (None, vec![]),
            |(mut default, mut conditions), property| {
                if let Some(condition) = property.condition {
                    conditions.push(ftd::interpreter::ConditionalValue::new(
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
                fastn_type::PropertyValue::Value {
                    value: fastn_type::Value::Optional {
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

        let self_reference = get_self_reference(&default, component_name);

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

                return Ok(Some((
                    name_in_component_definition,
                    name.to_string(),
                    vec![],
                )));
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

        let variable = ftd::interpreter::Variable {
            name: variable_name.to_string(),
            kind: argument.kind.to_owned(),
            mutable: argument.mutable,
            value: default,
            conditional_value: conditions,
            line_number,
            is_static: true,
        }
        .set_static(&self.itdoc());

        ftd::interpreter::utils::validate_variable(&variable, &self.itdoc())?;

        self.bag.insert(
            variable.name.to_string(),
            ftd::interpreter::Thing::Variable(variable),
        );

        Ok(Some((
            name_in_component_definition,
            variable_name,
            self_reference,
        )))
    }
}

fn get_self_reference(default: &fastn_type::PropertyValue, component_name: &str) -> Vec<String> {
    match default {
        fastn_type::PropertyValue::Reference { name, .. }
        | fastn_type::PropertyValue::Clone { name, .. }
            if name.starts_with(format!("{}.", component_name).as_str()) =>
        {
            vec![name.to_string()]
        }
        fastn_type::PropertyValue::FunctionCall(f) => {
            let mut self_reference = vec![];
            for arguments in f.values.values() {
                self_reference.extend(get_self_reference(arguments, component_name));
            }
            self_reference
        }
        _ => vec![],
    }
}

fn set_reference_name(default: &mut fastn_type::PropertyValue, values: &ftd::Map<String>) {
    match default {
        fastn_type::PropertyValue::Reference { name, .. }
        | fastn_type::PropertyValue::Clone { name, .. } => {
            *name = values.get(name).unwrap().to_string();
        }
        fastn_type::PropertyValue::FunctionCall(f) => {
            for arguments in f.values.values_mut() {
                set_reference_name(arguments, values);
            }
        }
        _ => {}
    }
}
