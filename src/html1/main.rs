#![allow(dead_code)]

pub struct HtmlUI {
    pub html: String,
    pub dependencies: String,
    pub variables: String,
    pub functions: String,
}

impl HtmlUI {
    pub fn from_node_data(node_data: ftd::node::NodeData, id: &str) -> HtmlUI {
        let functions = ftd::html1::FunctionGenerator::new(id).get_functions(&node_data);
        let dependencies = ftd::html1::dependencies::DependencyGenerator::new(id, &node_data.node)
            .get_dependencies();
        let html = HtmlGenerator::new(id).to_html(node_data.name.as_str(), node_data.node);
        let variables =
            ftd::html1::data::DataGenerator::new(node_data.name.as_str(), &node_data.bag)
                .get_data();

        HtmlUI {
            html,
            dependencies,
            variables: serde_json::to_string_pretty(&variables)
                .expect("failed to convert document to json"),
            functions,
        }
    }
}

pub(crate) struct HtmlGenerator {
    pub id: String,
}

impl HtmlGenerator {
    pub fn new(id: &str) -> HtmlGenerator {
        HtmlGenerator { id: id.to_string() }
    }

    pub fn to_html(&self, doc_name: &str, node: ftd::node::Node) -> String {
        let style = format!(
            "style=\"{}\"",
            self.style_to_html(&node, /*self.visible*/ true)
        );
        let classes = self.class_to_html(&node);

        let attrs = {
            let mut attr = self.attrs_to_html(&node);
            let events = self.group_by_js_event(&node.events);
            for (name, actions) in events {
                let event = format!(
                    "window.ftd.handle_event(event, '{}', '{}', this)",
                    self.id,
                    actions.replace('\"', "&quot;")
                );
                attr.push(' ');
                attr.push_str(&format!("{}={}", name, quote(&event)));
            }
            attr
        };

        let body = match node.text.value.as_ref() {
            Some(v) => v.to_string(),
            None => node
                .children
                .into_iter()
                .map(|v| self.to_html(doc_name, v))
                .collect::<Vec<String>>()
                .join(""),
        };

        format!(
            "<{node} {attrs} {style} {classes}>{body}</{node}>",
            node = node.node.as_str(),
            attrs = attrs,
            style = style,
            classes = classes,
            body = body,
        )
    }

    pub fn style_to_html(&self, node: &ftd::node::Node, visible: bool) -> String {
        let mut styles: ftd::Map<String> = node
            .style
            .to_owned()
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
