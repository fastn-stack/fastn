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

    pub(crate) fn get_dependencies(&self) -> ftd::html1::Result<(String, ftd::VecMap<String>)> {
        let mut var_dependencies: ftd::VecMap<String> = Default::default();
        let dependencies = self.get_dependencies_(&mut var_dependencies)?;
        if dependencies.trim().is_empty() {
            return Ok(("".to_string(), Default::default()));
        }
        Ok((
            format!("window.node_change_{} = {{}};\n{}", self.id, dependencies),
            var_dependencies,
        ))
    }

    fn get_dependencies_(
        &self,
        var_dependencies: &mut ftd::VecMap<String>,
    ) -> ftd::html1::Result<String> {
        let node_data_id = ftd::html1::utils::full_data_id(self.id, self.node.data_id.as_str());
        let mut result = vec![];

        {
            let mut expressions = vec![];

            let node_change_id =
                ftd::html1::utils::node_change_id(node_data_id.as_str(), "display");
            dependency_map_from_condition(
                var_dependencies,
                &self.node.condition,
                node_change_id.as_str(),
            );

            let condition = self
                .node
                .condition
                .as_ref()
                .map(ftd::html1::utils::get_condition_string);

            let key = format!(
                "document.querySelector(`[data-id=\"{}\"]`).style[\"display\"]",
                node_data_id
            );

            if let Some(condition) = condition {
                let pos_value = format!("{} = \"flex\";", key);
                let neg_value = format!("{} = \"none\";", key);
                expressions.push((Some(condition), pos_value));
                expressions.push((None, neg_value));
            }

            let value = ftd::html1::utils::js_expression_from_list(expressions, Some(key.as_str()));
            if !value.trim().is_empty() {
                result.push(format!(
                    indoc::indoc! {"
                         window.node_change_{id}[\"{key}\"] = function(data) {{
                                {value}
                         }}
                    "},
                    id = self.id,
                    key = node_change_id,
                    value = value.trim(),
                ));
            }
        }

        {
            let node_change_id = ftd::html1::utils::node_change_id(node_data_id.as_str(), "text");
            let mut expressions = vec![];
            let mut is_static = true;
            let key = format!(
                "document.querySelector(`[data-id=\"{}\"]`).innerHTML",
                node_data_id,
            );
            for property_with_pattern in self.node.text.properties.iter() {
                let property = &property_with_pattern.property;
                let condition = property
                    .condition
                    .as_ref()
                    .map(ftd::html1::utils::get_condition_string);

                if !is_static_expression(&property.value, &condition) {
                    is_static = false;
                }

                if let Some(value_string) =
                    ftd::html1::utils::get_formatted_dep_string_from_property_value(
                        self.id,
                        self.doc,
                        &property.value,
                        &property_with_pattern.pattern_with_eval,
                        None,
                    )?
                {
                    dependency_map_from_condition(
                        var_dependencies,
                        &property.condition,
                        node_change_id.as_str(),
                    );
                    dependency_map_from_property_value(
                        var_dependencies,
                        &property.value,
                        node_change_id.as_str(),
                    );

                    let value = format!("{} = {};", key, value_string);
                    expressions.push((condition, value));
                }
            }
            let value = ftd::html1::utils::js_expression_from_list(expressions, Some(key.as_str()));
            if !value.trim().is_empty() && !is_static {
                result.push(format!(
                    indoc::indoc! {"
                         window.node_change_{id}[\"{key}\"] = function(data) {{
                                {value}
                         }}
                    "},
                    id = self.id,
                    key = node_change_id,
                    value = value.trim(),
                ));
            }
        }

        for (key, attribute) in self.node.attrs.iter() {
            let mut expressions = vec![];
            let mut is_static = true;
            let node_change_id = ftd::html1::utils::node_change_id(node_data_id.as_str(), key);
            for property_with_pattern in attribute.properties.iter() {
                let property = &property_with_pattern.property;
                let condition = property
                    .condition
                    .as_ref()
                    .map(ftd::html1::utils::get_condition_string);

                if !is_static_expression(&property.value, &condition) {
                    is_static = false;
                }

                if ftd::html1::utils::is_dark_mode_dependent(&property.value, self.doc)? {
                    // Todo: If the property.value is static then resolve it and use
                    /*let value = property
                        .value
                        .clone()
                        .resolve(&self.doc, property.value.line_number())?
                        .record_fields(self.doc.name, property.line_number)?;

                    let light = value.get("light").unwrap();
                    let dark = value.get("dark").unwrap();

                    if condition.is_none() && dark.eq(light) {
                        dbg!("condition.is_none()", &dark);
                        continue;
                    }*/
                    let mut expressions = vec![];
                    let mut light_value_string = "".to_string();
                    if let Some(value_string) =
                        ftd::html1::utils::get_formatted_dep_string_from_property_value(
                            self.id,
                            self.doc,
                            &property.value,
                            &property_with_pattern.pattern_with_eval,
                            Some("light".to_string()),
                        )?
                    {
                        let value = format!(
                            "document.querySelector(`[data-id=\"{}\"]`).setAttribute(\"{}\", {});",
                            node_data_id, key, value_string
                        );
                        let condition = Some(match condition {
                            Some(ref c) => format!("{} && !data[\"ftd#dark-mode\"]", c),
                            None => "!data[\"ftd#dark-mode\"]".to_string(),
                        });
                        expressions.push((condition, value));
                        light_value_string = value_string;
                    }

                    let mut dark_value_string = "".to_string();
                    if let Some(value_string) =
                        ftd::html1::utils::get_formatted_dep_string_from_property_value(
                            self.id,
                            self.doc,
                            &property.value,
                            &property_with_pattern.pattern_with_eval,
                            Some("dark".to_string()),
                        )?
                    {
                        let value = format!(
                            "document.querySelector(`[data-id=\"{}\"]`).setAttribute(\"{}\", {});",
                            node_data_id, key, value_string
                        );
                        let condition = Some(match condition {
                            Some(ref c) => format!("{} && data[\"ftd#dark-mode\"]", c),
                            None => "data[\"ftd#dark-mode\"]".to_string(),
                        });
                        expressions.push((condition, value));
                        dark_value_string = value_string;
                    }

                    if !light_value_string.eq(&dark_value_string) {
                        let value = ftd::html1::utils::js_expression_from_list(expressions, None);
                        if !value.trim().is_empty() {
                            dependency_map_from_condition(
                                var_dependencies,
                                &property.condition,
                                node_change_id.as_str(),
                            );
                            dependency_map_from_property_value(
                                var_dependencies,
                                &property.value,
                                node_change_id.as_str(),
                            );
                            var_dependencies
                                .insert("ftd#dark-mode".to_string(), node_change_id.to_string());
                            result.push(format!(
                                indoc::indoc! {"
                                     window.node_change_{id}[\"{key}\"] = function(data) {{
                                            {value}
                                     }}
                                "},
                                id = self.id,
                                key = node_change_id,
                                value = value.trim(),
                            ));
                        }
                    }
                    continue;
                }

                if let Some(value_string) =
                    ftd::html1::utils::get_formatted_dep_string_from_property_value(
                        self.id,
                        self.doc,
                        &property.value,
                        &property_with_pattern.pattern_with_eval,
                        None,
                    )?
                {
                    dependency_map_from_condition(
                        var_dependencies,
                        &property.condition,
                        node_change_id.as_str(),
                    );
                    dependency_map_from_property_value(
                        var_dependencies,
                        &property.value,
                        node_change_id.as_str(),
                    );
                    let value = format!(
                        "document.querySelector(`[data-id=\"{}\"]`).setAttribute(\"{}\", {});",
                        node_data_id, key, value_string
                    );
                    expressions.push((condition, value));
                }
            }
            let value = ftd::html1::utils::js_expression_from_list(expressions, None);
            if !value.trim().is_empty() && !is_static {
                result.push(format!(
                    indoc::indoc! {"
                         window.node_change_{id}[\"{key}\"] = function(data) {{
                                {value}
                         }}
                    "},
                    id = self.id,
                    key = node_change_id,
                    value = value.trim(),
                ));
            }
        }

        for (key, attribute) in self.node.style.iter() {
            let mut expressions = vec![];
            let mut is_static = true;
            let node_change_id = ftd::html1::utils::node_change_id(node_data_id.as_str(), key);
            let key = format!(
                "document.querySelector(`[data-id=\"{}\"]`).style[\"{}\"]",
                node_data_id, key
            );
            for property_with_pattern in attribute.properties.iter() {
                let property = &property_with_pattern.property;
                let condition = property
                    .condition
                    .as_ref()
                    .map(ftd::html1::utils::get_condition_string);

                if !is_static_expression(&property.value, &condition) {
                    is_static = false;
                }

                if let Some(value_string) =
                    ftd::html1::utils::get_formatted_dep_string_from_property_value(
                        self.id,
                        self.doc,
                        &property.value,
                        &property_with_pattern.pattern_with_eval,
                        None,
                    )?
                {
                    dependency_map_from_condition(
                        var_dependencies,
                        &property.condition,
                        node_change_id.as_str(),
                    );
                    dependency_map_from_property_value(
                        var_dependencies,
                        &property.value,
                        node_change_id.as_str(),
                    );
                    let value = format!("{} = {};", key, value_string);
                    expressions.push((condition, value));
                }
            }

            let value = ftd::html1::utils::js_expression_from_list(expressions, Some(key.as_str()));
            if !value.trim().is_empty() && !is_static {
                result.push(format!(
                    indoc::indoc! {"
                         window.node_change_{id}[\"{key}\"] = function(data) {{
                                {value}
                         }}
                    "},
                    id = self.id,
                    key = node_change_id,
                    value = value.trim(),
                ));
            }
        }

        for children in self.node.children.iter() {
            let value = DependencyGenerator::new(self.id, children, self.doc)
                .get_dependencies_(var_dependencies)?;
            if !value.trim().is_empty() {
                result.push(value.trim().to_string());
            }
        }
        Ok(result.join("\n"))
    }
}

fn dependency_map_from_condition(
    var_dependencies: &mut ftd::VecMap<String>,
    condition: &Option<ftd::interpreter2::Expression>,
    node_change_id: &str,
) {
    if let Some(condition) = condition.as_ref() {
        for reference in condition.references.values() {
            dependency_map_from_property_value(var_dependencies, reference, node_change_id)
        }
    }
}

fn dependency_map_from_property_value(
    var_dependencies: &mut ftd::VecMap<String>,
    property_value: &ftd::interpreter2::PropertyValue,
    node_change_id: &str,
) {
    let values = ftd::html1::utils::dependencies_from_property_value(property_value);
    for v in values {
        var_dependencies.insert(v, node_change_id.to_string());
    }
}

fn is_static_expression(
    property_value: &ftd::interpreter2::PropertyValue,
    condition: &Option<String>,
) -> bool {
    if property_value.kind().is_ftd_length() {
        if let ftd::interpreter2::PropertyValue::Value {
            value: ftd::interpreter2::Value::OrType { fields, .. },
            ..
        } = property_value
        {
            if !fields
                .get(ftd::interpreter2::FTD_LENGTH_VALUE)
                .map(|v| v.is_value())
                .unwrap_or(true)
            {
                return false;
            }
        }
    }
    if condition.is_some() || !property_value.is_value() {
        return false;
    }

    true
}
