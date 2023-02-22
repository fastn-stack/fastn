#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: &'a mut ftd::Map<ftd::interpreter2::Thing>,
    pub dummy_instructions: &'a mut ftd::Map<ftd::executor::DummyElement>,
    pub element_constructor: &'a mut ftd::Map<ftd::executor::ElementConstructor>,
    pub js: &'a mut std::collections::HashSet<String>,
    pub css: &'a mut std::collections::HashSet<String>,
}

impl<'a> TDoc<'a> {
    pub(crate) fn itdoc(&self) -> ftd::interpreter2::TDoc {
        ftd::interpreter2::TDoc::new(self.name, self.aliases, self.bag)
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
        let mut map: ftd::Map<String> = Default::default();
        for argument in arguments {
            if let Some((k, v)) = self.insert_local_variable(
                component_name,
                properties,
                argument,
                container,
                line_number,
                inherited_variables,
                insert_null,
            )? {
                map.insert(k, v);
            }
        }
        Ok(map)
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
    ) -> ftd::executor::Result<Option<(String, String)>> {
        let string_container = ftd::executor::utils::get_string_container(container);
        let source = argument.to_sources();
        let properties = ftd::executor::value::find_properties_by_source(
            source.as_slice(),
            properties,
            self,
            argument,
            line_number,
        )?;

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

        let name_in_component_definition = format!("{}.{}", component_name, argument.name);
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

                return Ok(Some((name_in_component_definition, name.to_string())));
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

        Ok(Some((name_in_component_definition, variable_name)))
    }
}
