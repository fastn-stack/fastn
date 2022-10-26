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
        for children in self.node.children.iter() {
            result.push(DependencyGenerator::new(self.id, children).get_dependencies_());
        }
        result.join("\n\n")
    }
}
