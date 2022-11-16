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

        {
            let mut expressions = vec![];

            let condition = self
                .node
                .condition
                .as_ref()
                .map(ftd::html1::utils::get_condition_string);

            if let Some(condition) = condition {
                let pos_value = format!(
                    "document.querySelector(`[data-id=\"{}\"]`).style[\"display\"] = \"flex\";",
                    node_data_id
                );
                let neg_value = format!(
                    "document.querySelector(`[data-id=\"{}\"]`).style[\"display\"] = \"none\";",
                    node_data_id
                );
                expressions.push((Some(condition), pos_value));
                expressions.push((None, neg_value));
            }

            let value = ftd::html1::utils::js_expression_from_list(expressions);
            if !value.trim().is_empty() {
                result.push(value.trim().to_string());
            }
        }

        {
            let mut expressions = vec![];
            for property in self.node.text.properties.iter() {
                let condition = property
                    .condition
                    .as_ref()
                    .map(ftd::html1::utils::get_condition_string);
                if let Some(value_string) =
                    ftd::html1::utils::get_formatted_dep_string_from_property_value(
                        self.id,
                        self.doc,
                        &property.value,
                        &self.node.text.pattern,
                        None,
                    )?
                {
                    let value = format!(
                        "document.querySelector(`[data-id=\"{}\"]`).innerHTML = {};",
                        node_data_id, value_string
                    );
                    expressions.push((condition, value));
                }
            }
            let value = ftd::html1::utils::js_expression_from_list(expressions);
            if !value.trim().is_empty() {
                result.push(value.trim().to_string());
            }
        }

        for (key, attribute) in self.node.attrs.iter() {
            let mut expressions = vec![];
            for property in attribute.properties.iter() {
                let condition = property
                    .condition
                    .as_ref()
                    .map(ftd::html1::utils::get_condition_string);

                if ftd::html1::utils::is_dark_mode_dependent(&property.value, &self.doc)? {
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
                            &attribute.pattern,
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
                            &attribute.pattern,
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
                        let value = ftd::html1::utils::js_expression_from_list(expressions);
                        if !value.trim().is_empty() {
                            result.push(value.trim().to_string());
                        }
                    }
                    continue;
                }

                if let Some(value_string) =
                    ftd::html1::utils::get_formatted_dep_string_from_property_value(
                        self.id,
                        self.doc,
                        &property.value,
                        &attribute.pattern,
                        None,
                    )?
                {
                    let value = format!(
                        "document.querySelector(`[data-id=\"{}\"]`).setAttribute(\"{}\", {});",
                        node_data_id, key, value_string
                    );
                    expressions.push((condition, value));
                }
            }
            let value = ftd::html1::utils::js_expression_from_list(expressions);
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
                    .map(ftd::html1::utils::get_condition_string);
                if let Some(value_string) =
                    ftd::html1::utils::get_formatted_dep_string_from_property_value(
                        self.id,
                        self.doc,
                        &property.value,
                        &attribute.pattern,
                        None,
                    )?
                {
                    let value = format!(
                        "document.querySelector(`[data-id=\"{}\"]`).style[\"{}\"] = {};",
                        node_data_id, key, value_string
                    );
                    expressions.push((condition, value));
                }
            }

            let value = ftd::html1::utils::js_expression_from_list(expressions);
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
}
