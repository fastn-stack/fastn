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
                self.doc,
            );

            let condition = self
                .node
                .condition
                .as_ref()
                .map(|c| ftd::html1::utils::get_condition_string_(c, false));

            let key = format!(
                "document.querySelector(`[data-id=\"{}\"]`).style[\"display\"]",
                node_data_id
            );

            if let Some(condition) = condition {
                let pos_value = format!("{} = \"{}\";", key, self.node.display);
                let neg_value = format!("{} = \"none\";", key);
                expressions.push((Some(condition), pos_value));
                expressions.push((None, neg_value));
            }

            let value = ftd::html1::utils::js_expression_from_list(
                expressions,
                Some(key.as_str()),
                format!("{} = null;", key).as_str(),
            );
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
                    .map(|c| ftd::html1::utils::get_condition_string_(c, false));

                if !is_static_expression(&property.value, &condition, self.doc) {
                    is_static = false;
                }

                if let Some(value_string) =
                    ftd::html1::utils::get_formatted_dep_string_from_property_value(
                        self.id,
                        self.doc,
                        &property.value,
                        &property_with_pattern.pattern_with_eval,
                        None,
                        false,
                    )?
                {
                    dependency_map_from_condition(
                        var_dependencies,
                        &property.condition,
                        node_change_id.as_str(),
                        self.doc,
                    );
                    dependency_map_from_property_value(
                        var_dependencies,
                        &property.value,
                        node_change_id.as_str(),
                        self.doc,
                    );

                    let value = format!("{} = {};", key, value_string);
                    expressions.push((condition, value));
                }
            }
            let value = ftd::html1::utils::js_expression_from_list(
                expressions,
                Some(key.as_str()),
                format!(
                    "{} = {}",
                    key,
                    self.node
                        .text
                        .default
                        .clone()
                        .unwrap_or_else(|| "null".to_string())
                )
                .as_str(),
            );
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
                    .map(|c| ftd::html1::utils::get_condition_string_(c, false));

                if !is_static_expression(&property.value, &condition, self.doc) {
                    is_static = false;
                }

                if ftd::html1::utils::is_device_dependent(&property.value, self.doc)? {
                    let mut desktop_value_string = "".to_string();
                    if let Some(value_string) =
                        ftd::html1::utils::get_formatted_dep_string_from_property_value(
                            self.id,
                            self.doc,
                            &property.value,
                            &property_with_pattern.pattern_with_eval,
                            Some("desktop".to_string()),
                            false,
                        )?
                    {
                        desktop_value_string = value_string;
                    }

                    let mut mobile_value_string = "".to_string();
                    if let Some(value_string) =
                        ftd::html1::utils::get_formatted_dep_string_from_property_value(
                            self.id,
                            self.doc,
                            &property.value,
                            &property_with_pattern.pattern_with_eval,
                            Some("mobile".to_string()),
                            false,
                        )?
                    {
                        mobile_value_string = value_string;
                    }
                    if desktop_value_string.ne(&mobile_value_string) {
                        is_static = false;
                        let value = ftd::html1::utils::js_expression_from_list(
                            std::iter::IntoIterator::into_iter([
                                (
                                    Some("data[\"ftd#device\"] == \"desktop\"".to_string()),
                                    format!(
                                        "document.querySelector(`[data-id=\"{}\"]`).setAttribute(\"{}\", {});",
                                        node_data_id, key, desktop_value_string
                                    )
                                ),
                                (None, format!(
                                    "document.querySelector(`[data-id=\"{}\"]`).setAttribute(\"{}\", {});",
                                    node_data_id, key, mobile_value_string
                                )),
                            ])
                                .collect(),
                            Some(key.as_str()),attribute
                                .default
                                .as_ref()
                                .map(|v| {
                                    format!(
                                        "document.querySelector(`[data-id=\"{}\"]`).setAttribute(\"{}\", {});",
                                        node_data_id, key, v
                                    )
                                })
                                .unwrap_or_else(|| {
                                    format!(
                                        "document.querySelector(`[data-id=\"{}\"]`).removeAttribute(\"{}\");",
                                        node_data_id, key,
                                    )
                                })
                                .as_str(),
                        );
                        expressions.push((condition, value));
                    } else {
                        expressions
                            .push((condition, format!("{} = {};", key, desktop_value_string)));
                    }

                    if !desktop_value_string.is_empty() {
                        dependency_map_from_condition(
                            var_dependencies,
                            &property.condition,
                            node_change_id.as_str(),
                            self.doc,
                        );
                        dependency_map_from_property_value(
                            var_dependencies,
                            &property.value,
                            node_change_id.as_str(),
                            self.doc,
                        );
                        var_dependencies
                            .insert("ftd#device".to_string(), node_change_id.to_string());
                    }
                    continue;
                }

                if ftd::html1::utils::is_dark_mode_dependent(&property.value, self.doc)? {
                    // Todo: If the property.value is static then resolve it and use
                    let mut light_value_string = "".to_string();
                    if let Some(value_string) =
                        ftd::html1::utils::get_formatted_dep_string_from_property_value(
                            self.id,
                            self.doc,
                            &property.value,
                            &property_with_pattern.pattern_with_eval,
                            Some("light".to_string()),
                            false,
                        )?
                    {
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
                            false,
                        )?
                    {
                        dark_value_string = value_string;
                    }

                    if light_value_string.ne(&dark_value_string) {
                        is_static = false;
                        let value = ftd::html1::utils::js_expression_from_list(
                            std::iter::IntoIterator::into_iter([
                                (
                                    Some("!data[\"ftd#dark-mode\"]".to_string()),
                                    format!(
                                        "document.querySelector(`[data-id=\"{}\"]`).setAttribute(\"{}\", {});",
                                        node_data_id, key, light_value_string
                                    )
                                ),
                                (None, format!(
                                    "document.querySelector(`[data-id=\"{}\"]`).setAttribute(\"{}\", {});",
                                    node_data_id, key, dark_value_string
                                )),
                            ])
                            .collect(),
                            Some(key.as_str()),attribute
                                .default
                                .as_ref()
                                .map(|v| {
                                    format!(
                                        "document.querySelector(`[data-id=\"{}\"]`).setAttribute(\"{}\", {});",
                                        node_data_id, key, v
                                    )
                                })
                                .unwrap_or_else(|| {
                                    format!(
                                        "document.querySelector(`[data-id=\"{}\"]`).removeAttribute(\"{}\");",
                                        node_data_id, key,
                                    )
                                })
                                .as_str(),
                        );
                        expressions.push((condition, value));
                    } else {
                        expressions.push((condition, format!("{} = {};", key, light_value_string)));
                    }

                    if !light_value_string.is_empty() {
                        dependency_map_from_condition(
                            var_dependencies,
                            &property.condition,
                            node_change_id.as_str(),
                            self.doc,
                        );
                        dependency_map_from_property_value(
                            var_dependencies,
                            &property.value,
                            node_change_id.as_str(),
                            self.doc,
                        );
                        var_dependencies
                            .insert("ftd#dark-mode".to_string(), node_change_id.to_string());
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
                        key.eq("class"),
                    )?
                {
                    dependency_map_from_condition(
                        var_dependencies,
                        &property.condition,
                        node_change_id.as_str(),
                        self.doc,
                    );
                    dependency_map_from_property_value(
                        var_dependencies,
                        &property.value,
                        node_change_id.as_str(),
                        self.doc,
                    );
                    let value = format!(
                        "document.querySelector(`[data-id=\"{}\"]`).setAttribute(\"{}\", {});",
                        node_data_id, key, value_string
                    );
                    expressions.push((condition, value));
                }
            }
            let mut value = ftd::html1::utils::js_expression_from_list(
                expressions,
                Some(key),
                attribute
                    .default
                    .as_ref()
                    .map(|v| {
                        format!(
                            "document.querySelector(`[data-id=\"{}\"]`).setAttribute(\"{}\", {});",
                            node_data_id, key, v
                        )
                    })
                    .unwrap_or_else(|| {
                        format!(
                            "document.querySelector(`[data-id=\"{}\"]`).removeAttribute(\"{}\");",
                            node_data_id, key,
                        )
                    })
                    .as_str(),
            );

            let remove_case_condition = format!(
                indoc::indoc! {"
                if (document.querySelector(`[data-id=\"{}\"]`).getAttribute(\"{}\") == \"{}\"){{
                    document.querySelector(`[data-id=\"{}\"]`).removeAttribute(\"{}\");
                }}
            "},
                node_data_id,
                key,
                ftd::interpreter2::FTD_REMOVE_KEY,
                node_data_id,
                key
            );

            value = format!("{}\n{}", value, remove_case_condition);

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
                    .map(|c| ftd::html1::utils::get_condition_string_(c, false));

                if !is_static_expression(&property.value, &condition, self.doc) {
                    is_static = false;
                }

                if ftd::html1::utils::is_device_dependent(&property.value, self.doc)? {
                    let mut desktop_value_string = "".to_string();
                    if let Some(value_string) =
                        ftd::html1::utils::get_formatted_dep_string_from_property_value(
                            self.id,
                            self.doc,
                            &property.value,
                            &property_with_pattern.pattern_with_eval,
                            Some("desktop".to_string()),
                            false,
                        )?
                    {
                        desktop_value_string = value_string;
                    }

                    let mut mobile_value_string = "".to_string();
                    if let Some(value_string) =
                        ftd::html1::utils::get_formatted_dep_string_from_property_value(
                            self.id,
                            self.doc,
                            &property.value,
                            &property_with_pattern.pattern_with_eval,
                            Some("mobile".to_string()),
                            false,
                        )?
                    {
                        mobile_value_string = value_string;
                    }
                    if desktop_value_string.ne(&mobile_value_string) {
                        is_static = false;
                        let value = ftd::html1::utils::js_expression_from_list(
                            std::iter::IntoIterator::into_iter([
                                (
                                    Some("data[\"ftd#device\"] == \"desktop\"".to_string()),
                                    format!("{} = {};", key, desktop_value_string),
                                ),
                                (None, format!("{} = {};", key, mobile_value_string)),
                            ])
                            .collect(),
                            Some(key.as_str()),
                            format!(
                                "{} = {}",
                                key,
                                attribute
                                    .default
                                    .clone()
                                    .unwrap_or_else(|| "null".to_string())
                            )
                            .as_str(),
                        );
                        expressions.push((condition, value));
                    } else {
                        expressions
                            .push((condition, format!("{} = {};", key, desktop_value_string)));
                    }

                    if !desktop_value_string.is_empty() {
                        dependency_map_from_condition(
                            var_dependencies,
                            &property.condition,
                            node_change_id.as_str(),
                            self.doc,
                        );
                        dependency_map_from_property_value(
                            var_dependencies,
                            &property.value,
                            node_change_id.as_str(),
                            self.doc,
                        );
                        var_dependencies
                            .insert("ftd#device".to_string(), node_change_id.to_string());
                    }
                    continue;
                }

                if ftd::html1::utils::is_dark_mode_dependent(&property.value, self.doc)? {
                    // Todo: If the property.value is static then resolve it and use
                    let mut light_value_string = "".to_string();
                    if let Some(value_string) =
                        ftd::html1::utils::get_formatted_dep_string_from_property_value(
                            self.id,
                            self.doc,
                            &property.value,
                            &property_with_pattern.pattern_with_eval,
                            Some("light".to_string()),
                            false,
                        )?
                    {
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
                            false,
                        )?
                    {
                        dark_value_string = value_string;
                    }

                    if light_value_string.ne(&dark_value_string) {
                        is_static = false;
                        let value = ftd::html1::utils::js_expression_from_list(
                            std::iter::IntoIterator::into_iter([
                                (
                                    Some("!data[\"ftd#dark-mode\"]".to_string()),
                                    format!("{} = {};", key, light_value_string),
                                ),
                                (None, format!("{} = {};", key, dark_value_string)),
                            ])
                            .collect(),
                            Some(key.as_str()),
                            format!(
                                "{} = {}",
                                key,
                                attribute
                                    .default
                                    .clone()
                                    .unwrap_or_else(|| "null".to_string())
                            )
                            .as_str(),
                        );
                        expressions.push((condition, value));
                    } else {
                        expressions.push((condition, format!("{} = {};", key, light_value_string)));
                    }

                    if !light_value_string.is_empty() {
                        dependency_map_from_condition(
                            var_dependencies,
                            &property.condition,
                            node_change_id.as_str(),
                            self.doc,
                        );
                        dependency_map_from_property_value(
                            var_dependencies,
                            &property.value,
                            node_change_id.as_str(),
                            self.doc,
                        );
                        var_dependencies
                            .insert("ftd#dark-mode".to_string(), node_change_id.to_string());
                    }

                    /*let mut light_value_string = "".to_string();
                    if let Some(value_string) =
                        ftd::html1::utils::get_formatted_dep_string_from_property_value(
                            self.id,
                            self.doc,
                            &property.value,
                            &property_with_pattern.pattern_with_eval,
                            Some("light".to_string()),
                            false,
                        )?
                    {
                        let value = format!("{} = {};", key, value_string);
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
                            false,
                        )?
                    {
                        let value = format!("{} = {};", key, value_string);
                        let condition = Some(match condition {
                            Some(ref c) => format!("{} && data[\"ftd#dark-mode\"]", c),
                            None => "data[\"ftd#dark-mode\"]".to_string(),
                        });
                        expressions.push((condition, value));
                        dark_value_string = value_string;
                    }

                    if !light_value_string.eq(&dark_value_string)
                        || condition.is_some()
                        || length > 0
                    {
                        dependency_map_from_condition(
                            var_dependencies,
                            &property.condition,
                            node_change_id.as_str(),
                            self.doc,
                        );
                        dependency_map_from_property_value(
                            var_dependencies,
                            &property.value,
                            node_change_id.as_str(),
                            self.doc,
                        );
                        var_dependencies
                            .insert("ftd#dark-mode".to_string(), node_change_id.to_string());
                    }*/
                    continue;
                }

                if let Some(value_string) =
                    ftd::html1::utils::get_formatted_dep_string_from_property_value(
                        self.id,
                        self.doc,
                        &property.value,
                        &property_with_pattern.pattern_with_eval,
                        None,
                        false,
                    )?
                {
                    dependency_map_from_condition(
                        var_dependencies,
                        &property.condition,
                        node_change_id.as_str(),
                        self.doc,
                    );
                    dependency_map_from_property_value(
                        var_dependencies,
                        &property.value,
                        node_change_id.as_str(),
                        self.doc,
                    );
                    let value = format!("{} = {};", key, value_string);
                    expressions.push((condition, value));
                }
            }

            let value = ftd::html1::utils::js_expression_from_list(
                expressions,
                Some(key.as_str()),
                format!(
                    "{} = {}",
                    key,
                    attribute
                        .default
                        .clone()
                        .unwrap_or_else(|| "null".to_string())
                )
                .as_str(),
            );
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
    doc: &ftd::interpreter2::TDoc,
) {
    if let Some(condition) = condition.as_ref() {
        for reference in condition.references.values() {
            dependency_map_from_property_value(var_dependencies, reference, node_change_id, doc)
        }
    }
}

fn dependency_map_from_property_value(
    var_dependencies: &mut ftd::VecMap<String>,
    property_value: &ftd::interpreter2::PropertyValue,
    node_change_id: &str,
    doc: &ftd::interpreter2::TDoc,
) {
    let values = ftd::html1::utils::dependencies_from_property_value(property_value, doc);
    for v in values {
        var_dependencies.insert(v, node_change_id.to_string());
    }
}

fn is_static_expression(
    property_value: &ftd::interpreter2::PropertyValue,
    condition: &Option<String>,
    doc: &ftd::interpreter2::TDoc,
) -> bool {
    if property_value.kind().is_ftd_length() {
        if let ftd::interpreter2::PropertyValue::Value {
            value, line_number, ..
        } = property_value
        {
            if !value
                .get_or_type(doc.name, *line_number)
                .map(|v| v.2.is_value())
                .unwrap_or(false)
            {
                return false;
            }
        }
    }

    if property_value.kind().is_ftd_resizing() {
        if let ftd::interpreter2::PropertyValue::Value {
            value, line_number, ..
        } = property_value
        {
            let property_value = value.get_or_type(doc.name, *line_number).unwrap().2;
            if property_value.kind().is_ftd_length() {
                if let ftd::interpreter2::PropertyValue::Value {
                    value, line_number, ..
                } = property_value
                {
                    if !value
                        .get_or_type(doc.name, *line_number)
                        .map(|v| v.2.is_value())
                        .unwrap_or(false)
                    {
                        return false;
                    }
                }
            }
        }
    }

    if condition.is_some() || !property_value.is_value() {
        return false;
    }

    true
}
