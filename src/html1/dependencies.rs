pub struct DependencyGenerator<'a> {
    pub id: &'a str,
    pub node: &'a ftd::node::Node,
    pub doc: &'a ftd::interpreter2::TDoc<'a>,
}

impl<'a> DependencyGenerator<'a> {
    pub(crate) fn new(
        id: &'a str,
        node: &'a ftd::node::Node,
        doc: &'a ftd::interpreter2::TDoc,
    ) -> DependencyGenerator<'a> {
        DependencyGenerator { id, node, doc }
    }

    pub(crate) fn get_dependencies(&self) -> ftd::html1::Result<String> {
        let dependencies = self.get_dependencies_()?;
        if dependencies.trim().is_empty() {
            return Ok("".to_string());
        }
        Ok(format!(
            indoc::indoc! {"
                            function node_change_{id}(data){{
                                {dependencies}
                            }}
        
                        "},
            id = self.id,
            dependencies = dependencies.trim(),
        ))
    }

    fn get_dependencies_(&self) -> ftd::html1::Result<String> {
        let node_data_id = ftd::html1::utils::full_data_id(self.id, self.node.data_id.as_str());
        let mut result = vec![];

        let mut expressions = vec![];
        for property in self.node.text.properties.iter() {
            let condition = property
                .condition
                .as_ref()
                .map(|v| self.get_condition_string(v));
            if let Some(value_string) = self.get_formatted_dep_string_from_property_value(
                &property.value,
                &self.node.text.pattern,
            )? {
                let value = format!(
                    "document.querySelector(`[data-id=\"{}\"]`).innerHTML = {};",
                    node_data_id, value_string
                );
                expressions.push((condition, value));
            }
        }

        let value = js_expression_from_list(expressions);
        if !value.trim().is_empty() {
            result.push(value.trim().to_string());
        }

        for (key, attribute) in self.node.attrs.iter() {
            let mut expressions = vec![];
            for property in attribute.properties.iter() {
                let condition = property
                    .condition
                    .as_ref()
                    .map(|v| self.get_condition_string(v));
                if let Some(value_string) = self.get_formatted_dep_string_from_property_value(
                    &property.value,
                    &attribute.pattern,
                )? {
                    let value = format!(
                        "document.querySelector(`[data-id=\"{}\"]`).setAttribute(\"{}\", {});",
                        node_data_id, key, value_string
                    );
                    expressions.push((condition, value));
                }
            }
            let value = js_expression_from_list(expressions);
            if !value.trim().is_empty() {
                result.push(value.trim().to_string());
            }
        }

        for (key, attribute) in self.node.style.iter() {
            let mut expressions = vec![];
            for property in attribute.properties.iter() {
                let condition = property
                    .condition
                    .as_ref()
                    .map(|v| self.get_condition_string(v));
                if let Some(value_string) = self.get_formatted_dep_string_from_property_value(
                    &property.value,
                    &attribute.pattern,
                )? {
                    let value = format!(
                        "document.querySelector(`[data-id=\"{}\"]`).style[\"{}\"] = {};",
                        node_data_id, key, value_string
                    );
                    expressions.push((condition, value));
                }
            }

            let value = js_expression_from_list(expressions);
            if !value.trim().is_empty() {
                result.push(value.trim().to_string());
            }
        }

        for children in self.node.children.iter() {
            let value =
                DependencyGenerator::new(self.id, children, self.doc).get_dependencies_()?;
            if !value.trim().is_empty() {
                result.push(value.trim().to_string());
            }
        }
        Ok(result.join("\n"))
    }

    fn get_condition_string(&self, condition: &ftd::interpreter2::Boolean) -> String {
        let node = condition
            .expression
            .update_node_with_variable_reference(&condition.references);
        let expression = ftd::html1::ExpressionGenerator.to_string(&node, true, &[]);
        format!(
            indoc::indoc! {"
                function(){{
                    {expression}
                }}()"
            },
            expression = expression.trim(),
        )
    }

    fn get_formatted_dep_string_from_property_value(
        &self,
        property_value: &ftd::interpreter2::PropertyValue,
        pattern: &Option<String>,
    ) -> ftd::html1::Result<Option<String>> {
        let value_string = match property_value {
            ftd::interpreter2::PropertyValue::Reference { name, .. } => {
                format!("data[\"{}\"]", name)
            }
            ftd::interpreter2::PropertyValue::FunctionCall(function_call) => {
                let action = serde_json::to_string(&ftd::html1::Action::from_function_call(
                    function_call,
                    self.id,
                    self.doc,
                )?)
                .unwrap();
                format!(
                    "window.ftd.handle_function(event, '{}', '{}', this)",
                    self.id, action
                )
            }
            ftd::interpreter2::PropertyValue::Value {
                value, line_number, ..
            } => value.to_string(self.doc, *line_number)?,
            _ => return Ok(None),
        };

        Ok(Some(match pattern {
            Some(p) => format!("\"{}\".format({})", p, value_string),
            None => value_string,
        }))
    }
}

fn js_expression_from_list(expressions: Vec<(Option<String>, String)>) -> String {
    let mut conditions = vec![];
    let mut default = None;
    for (condition, expression) in expressions {
        if let Some(condition) = condition {
            conditions.push(format!(
                indoc::indoc! {"
                        {if_exp}({condition}){{
                            {expression}
                        }}
                    "},
                if_exp = if conditions.is_empty() {
                    "if"
                } else {
                    "else if"
                },
                condition = condition,
                expression = expression.trim(),
            ));
        } else {
            default = Some(expression)
        }
    }

    let default = match default {
        Some(d) if conditions.is_empty() => d,
        Some(d) => format!("else {{{}}}", d),
        None => "".to_string(),
    };

    format!(
        indoc::indoc! {"
            {expressions}{default}
        "},
        expressions = conditions.join(" "),
        default = default,
    )
}
