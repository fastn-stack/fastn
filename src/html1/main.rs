#![allow(dead_code)]

pub struct HtmlUI {
    pub html: String,
    pub dependencies: String,
    pub variables: String,
    pub functions: String,
    pub variable_dependencies: String,
    pub outer_events: String,
}

impl HtmlUI {
    #[tracing::instrument(skip_all)]
    pub fn from_node_data(node_data: ftd::node::NodeData, id: &str) -> ftd::html1::Result<HtmlUI> {
        let tdoc = ftd::interpreter2::TDoc::new(
            node_data.name.as_str(),
            &node_data.aliases,
            &node_data.bag,
        );

        let functions = ftd::html1::FunctionGenerator::new(id).get_functions(&node_data)?;
        let (dependencies, var_dependencies) =
            ftd::html1::dependencies::DependencyGenerator::new(id, &node_data.node, &tdoc)
                .get_dependencies()?;
        let variable_dependencies = ftd::html1::VariableDependencyGenerator::new(id, &tdoc)
            .get_set_functions(&var_dependencies)?;
        let variables = ftd::html1::data::DataGenerator::new(&tdoc).get_data()?;
        let (html, outer_events) =
            HtmlGenerator::new(id, &tdoc).to_html_and_outer_events(node_data.node)?;

        Ok(HtmlUI {
            html,
            dependencies,
            variables: serde_json::to_string_pretty(&variables)
                .expect("failed to convert document to json"),
            functions,
            variable_dependencies,
            outer_events,
        })
    }
}

pub(crate) struct HtmlGenerator<'a> {
    pub id: String,
    pub doc: &'a ftd::interpreter2::TDoc<'a>,
}

impl<'a> HtmlGenerator<'a> {
    pub fn new(id: &str, doc: &'a ftd::interpreter2::TDoc<'a>) -> HtmlGenerator<'a> {
        HtmlGenerator {
            id: id.to_string(),
            doc,
        }
    }

    pub fn to_html_and_outer_events(
        &self,
        node: ftd::node::Node,
    ) -> ftd::html1::Result<(String, String)> {
        let (html, events) = self.to_html_(node)?;
        Ok((html, ftd::html1::utils::events_to_string(events)))
    }

    #[allow(clippy::type_complexity)]
    pub fn to_html_(
        &self,
        node: ftd::node::Node,
    ) -> ftd::html1::Result<(String, Vec<(String, String, String)>)> {
        if node.is_null() {
            return Ok(("".to_string(), vec![]));
        }

        let style = format!(
            "style=\"{}\"",
            self.style_to_html(&node, /*self.visible*/ true)
        );
        let classes = self.class_to_html(&node);

        let mut outer_events = vec![];
        let attrs = {
            let mut attr = self.attrs_to_html(&node);
            let events = self.group_by_js_event(&node.events)?;
            for (name, actions) in events {
                if name.eq("onclickoutside") || name.starts_with("onglobalkey") {
                    let event = format!(
                        "window.ftd.handle_event(event, '{}', '{}', this)",
                        self.id, actions
                    );
                    outer_events.push((
                        ftd::html1::utils::full_data_id(self.id.as_str(), node.data_id.as_str()),
                        name,
                        event,
                    ));
                } else {
                    let event = format!(
                        "window.ftd.handle_event(event, '{}', '{}', this)",
                        self.id,
                        actions.replace('\"', "&quot;")
                    );
                    attr.push(' ');
                    attr.push_str(&format!("{}={}", name, quote(&event)));
                }
            }
            attr
        };

        let body = match node.text.value.as_ref() {
            Some(v) => v.to_string(),
            None => node
                .children
                .into_iter()
                .map(|v| match self.to_html_(v) {
                    Ok((html, events)) => {
                        outer_events.extend(events);
                        Ok(html)
                    }
                    Err(e) => Err(e),
                })
                .collect::<ftd::html1::Result<Vec<String>>>()?
                .join(""),
        };

        Ok((
            format!(
                "<{node} {attrs} {style} {classes}>{body}</{node}>",
                node = node.node.as_str(),
                attrs = attrs,
                style = style,
                classes = classes,
                body = body,
            ),
            outer_events,
        ))
    }

    pub fn style_to_html(&self, node: &ftd::node::Node, visible: bool) -> String {
        let mut styles: ftd::Map<String> = node
            .style
            .clone()
            .into_iter()
            .filter_map(|(k, v)| v.value.map(|v| (k, v)))
            .collect();
        if !visible {
            styles.insert("display".to_string(), "none".to_string());
        }
        styles
            .iter()
            .map(|(k, v)| format!("{}: {}", *k, escape(v))) // TODO: escape needed?
            .collect::<Vec<String>>()
            .join("; ")
    }

    pub fn class_to_html(&self, node: &ftd::node::Node) -> String {
        if node.classes.is_empty() {
            return "".to_string();
        }
        format!(
            "class=\"{}\"",
            node.classes
                .iter()
                .map(|k| k.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }

    fn attrs_to_html(&self, node: &ftd::node::Node) -> String {
        node.attrs
            .iter()
            .filter_map(|(k, v)| {
                if k.eq("class") {
                    return None;
                }
                v.value.as_ref().map(|v| {
                    let v = if k.eq("data-id") {
                        ftd::html1::utils::full_data_id(self.id.as_str(), v)
                    } else {
                        v.to_string()
                    };
                    format!("{}={}", *k, quote(v.as_str()))
                })
            }) // TODO: escape needed?
            .collect::<Vec<String>>()
            .join(" ")
    }
}

fn s(s: &str) -> String {
    s.to_string()
}

pub fn escape(s: &str) -> String {
    let s = s.replace('>', "\\u003E");
    let s = s.replace('<', "\\u003C");
    s.replace('&', "\\u0026")
}

fn quote(i: &str) -> String {
    format!("{:?}", i)
}
