pub(crate) struct DummyHtmlGenerator<'a> {
    pub id: String,
    pub doc: &'a ftd::interpreter2::TDoc<'a>,
}

impl<'a> DummyHtmlGenerator<'a> {
    pub fn new(id: &str, doc: &'a ftd::interpreter2::TDoc<'a>) -> DummyHtmlGenerator<'a> {
        DummyHtmlGenerator {
            id: id.to_string(),
            doc,
        }
    }

    pub fn from_dummy_nodes(&self, dummy_nodes: &ftd::Map<ftd::node::DummyNode>) -> String {
        let mut dummy_dependency = "".to_string();
        for (dependency, dummy_node) in dummy_nodes {
            dummy_dependency = format!(
                "{}\n{}",
                dummy_dependency,
                self.from_dummy_node(dummy_node, dependency)
            )
        }
        if dummy_dependency.trim().is_empty() {
            "".to_string()
        } else {
            format!(
                "window.append_data_{} = {{}};\n{}",
                self.id, dummy_dependency
            )
        }
    }

    pub fn from_dummy_node(&self, dummy_node: &ftd::node::DummyNode, dependency: &str) -> String {
        let dummy_html = ftd::html1::RawHtmlGenerator::from_node(
            self.id.as_str(),
            self.doc,
            dummy_node.main.to_owned(),
        );

        if let Some(iteration) = dummy_html.iteration {
            format!(
                indoc::indoc! {"
                    window.append_data_{id}[\"{dependency}\"] = function(all_data) {{
                        let list = resolve_reference(\"{dependency}\", all_data);
                        for (var i = 0; i < list.length; i++) {{
                            let new_data = {{
                                \"{alias}\": list[i],
                                \"LOOP__COUNTER\": i
                            }};
                            let data = {{...new_data, ...all_data}};
                            {arguments}
                            data = {{...args, ...all_data}};
                            if (!!\"{node}\".trim() && !!window[\"function_{id}\"] && !!window.function_{id}[\"{node}\"]) {{
                                data[\"{node}\"] = window.function_{id}[\"{node}\"](data);
                            }}
                            let html = \"{html}\".replace_format(data);
                            let nodes = stringToHTML(html).children;
                            let main = document.querySelector(`[data-id=\"{data_id}\"]`);
                            for (var child in nodes) {{
                                main.insertBefore(nodes[child], main.children[{start_index}]);
                            }}
                         }}
                    }}"
                },
                dependency = dependency,
                alias = iteration.alias,
                arguments = dummy_html.properties_string.unwrap_or_default(),
                node = dummy_html.name,
                html = dummy_html.html,
                id = self.id,
                data_id = ftd::html1::utils::full_data_id(
                    self.id.as_str(),
                    ftd::executor::utils::get_string_container(
                        dummy_node.parent_container.as_slice()
                    )
                    .as_str()
                ),
                start_index = dummy_node.start_index
            )
        } else {
            format!(
                indoc::indoc! {"
                    window.append_data_{id}[\"{dependency}\"] = function(all_data){{
                        {arguments}
                        let data = {{...args, ...all_data}};
                        if (!!\"{node}\".trim() && !!window[\"function_{id}\"] && !!window.function_{id}[\"{node}\"]) {{
                            data[\"{node}\"] = window.function_{id}[\"{node}\"](data);
                        }}
                        let html = \"{html}\".replace_format(data);
                        let nodes = stringToHTML(html).children;
                        let main = document.querySelector(`[data-id=\"{data_id}\"]`);
                        for (var child in nodes) {{
                            main.insertBefore(nodes[child], main.children[{start_index}]);
                        }}
                    }}"
                },
                dependency = dependency,
                arguments = dummy_html.properties_string.unwrap_or_default(),
                node = dummy_html.name,
                html = dummy_html.html,
                id = self.id,
                data_id = ftd::html1::utils::full_data_id(
                    self.id.as_str(),
                    ftd::executor::utils::get_string_container(
                        dummy_node.parent_container.as_slice()
                    )
                    .as_str()
                ),
                start_index = dummy_node.start_index
            )
        }
    }
}
