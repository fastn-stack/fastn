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
                .map(|v| ftd::html1::utils::get_condition_string(v));
            if let Some(value_string) =
                ftd::html1::utils::get_formatted_dep_string_from_property_value(
                    self.id,
                    self.doc,
                    &property.value,
                    &self.node.text.pattern,
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

        for (key, attribute) in self.node.attrs.iter() {
            let mut expressions = vec![];
            for property in attribute.properties.iter() {
                let condition = property
                    .condition
                    .as_ref()
                    .map(|v| ftd::html1::utils::get_condition_string(v));
                if let Some(value_string) =
                    ftd::html1::utils::get_formatted_dep_string_from_property_value(
                        self.id,
                        self.doc,
                        &property.value,
                        &attribute.pattern,
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
                    .map(|v| ftd::html1::utils::get_condition_string(v));
                if let Some(value_string) =
                    ftd::html1::utils::get_formatted_dep_string_from_property_value(
                        self.id,
                        self.doc,
                        &property.value,
                        &attribute.pattern,
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
