#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: &'a mut ftd::Map<ftd::interpreter2::Thing>,
    pub dummy_instructions: &'a mut ftd::Map<ftd::executor::dummy::DummyInstruction>,
    pub helper_instructions: &'a mut ftd::Map<ftd::executor::Element>,
}

impl<'a> TDoc<'a> {
    pub(crate) fn itdoc(&self) -> ftd::interpreter2::TDoc {
        ftd::interpreter2::TDoc::new(self.name, self.aliases, self.bag)
    }

    pub(crate) fn insert_local_variables(
        &mut self,
        component_name: &str,
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        container: &[usize],
        line_number: usize,
    ) -> ftd::executor::Result<ftd::Map<String>> {
        let string_container = ftd::executor::utils::get_string_container(container);
        let mut map: ftd::Map<String> = Default::default();
        for argument in arguments {
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
                        default = Some(property.value);
                    }
                    (default, conditions)
                },
            );

            let default = if let Some(default) = default {
                default
            } else {
                ftd::interpreter2::PropertyValue::Value {
                    value: ftd::interpreter2::Value::Optional {
                        data: Box::new(None),
                        kind: argument.kind.to_owned(),
                    },
                    is_mutable: argument.mutable,
                    line_number,
                }
            };

            let name_in_component_definition = format!("{}.{}", component_name, argument.name);
            match default.reference_name() {
                Some(name) if conditions.is_empty() => {
                    map.insert(name_in_component_definition, name.to_string());
                    continue;
                }
                _ => {}
            }

            let variable_name = self.itdoc().resolve_name(
                format!("{}:{}:{}", component_name, argument.name, string_container).as_str(),
            );
            map.insert(name_in_component_definition, variable_name.to_string());

            let variable = ftd::interpreter2::Variable {
                name: variable_name,
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
        }
        Ok(map)
    }
}
