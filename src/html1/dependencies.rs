pub struct DependencyGenerator<'a> {
    pub id: &'a str,
    pub node: &'a ftd::node::Node,
}

impl<'a> DependencyGenerator<'a> {
    pub(crate) fn new(id: &'a str, node: &'a ftd::node::Node) -> DependencyGenerator<'a> {
        DependencyGenerator { id, node }
    }

    pub(crate) fn get_dependencies(&self) -> String {
        let dependencies = self.get_dependencies_();
        if dependencies.trim().is_empty() {
            return "".to_string();
        }
        format!(
            indoc::indoc! {"
                    function node_change_{id}(data){{
                        {dependencies}
                    }}

                "},
            id = self.id,
            dependencies = dependencies.trim(),
        )
    }

    fn get_dependencies_(&self) -> String {
        let node_data_id = ftd::html1::utils::full_data_id(self.id, self.node.data_id.as_str());
        let mut result = vec![];
        let default = self
            .node
            .text
            .properties
            .iter()
            .find(|v| v.condition.is_none());
        if let Some(default) = default {
            if let ftd::interpreter2::PropertyValue::Reference { name, .. } = &default.value {
                result.push(format!(
                    "document.querySelector(`[data-id=\"{}\"]`).innerHTML = data[\"{}\"];",
                    node_data_id, name
                ))
            }
            // todo: else {}
        }

        for (key, attribute) in self.node.attrs.iter() {
            let default = attribute.properties.iter().find(|v| v.condition.is_none());
            if let Some(default) = default {
                if let ftd::interpreter2::PropertyValue::Reference { name, .. } = &default.value {
                    result.push(format!(
                        "document.querySelector(`[data-id=\"{}\"]`).setAttribute(\"{}\", data[\"{}\"]);",
                        node_data_id, key, name
                    ));
                }
                // todo: else {}
            }
        }

        for (key, attribute) in self.node.style.iter() {
            let default = attribute.properties.iter().find(|v| v.condition.is_none());
            if let Some(default) = default {
                if let ftd::interpreter2::PropertyValue::Reference { name, .. } = &default.value {
                    let value = if let Some(ref pattern) = attribute.pattern {
                        format!(
                            "document.querySelector(`[data-id=\"{}\"]`).style[\"{}\"] = \"{}\".format(data[\"{}\"]);",
                            node_data_id, key, pattern, name
                        )
                    } else {
                        format!(
                            "document.querySelector(`[data-id=\"{}\"]`).style[\"{}\"] = data[\"{}\"];",
                            node_data_id, key, name
                        )
                    };
                    result.push(value);
                } else if let ftd::interpreter2::PropertyValue::FunctionCall(function_call) =
                    &default.value
                {
                    let action = serde_json::to_string(&ftd::html1::Action::from_function_call(
                        &function_call,
                        self.id,
                    ))
                    .unwrap();
                    let event = format!(
                        "window.ftd.handle_function(event, '{}', '{}', this)",
                        self.id, action
                    );
                    let value = if let Some(ref pattern) = attribute.pattern {
                        format!(
                            "document.querySelector(`[data-id=\"{}\"]`).style[\"{}\"] = \"{}\".format({});",
                            node_data_id, key, pattern, event
                        )
                    } else {
                        format!(
                            "document.querySelector(`[data-id=\"{}\"]`).style[\"{}\"] = {};",
                            node_data_id, key, event
                        )
                    };
                    result.push(value);
                }
                // todo: else {}
            }
        }

        for children in self.node.children.iter() {
            result.push(DependencyGenerator::new(self.id, children).get_dependencies_());
        }
        result.join("\n\n")
    }
}
