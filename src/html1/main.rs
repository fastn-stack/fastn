#![allow(dead_code)]

pub struct HtmlUI {
    pub html: String,
    pub js: String,
    pub variables: String,
}

impl HtmlUI {
    pub fn from_node_data(node_data: ftd::node::NodeData) -> HtmlUI {
        let html = HtmlGenerator.to_html(node_data.node);
        HtmlUI {
            html,
            js: s(""),
            variables: s(""),
        }
    }
}

struct HtmlGenerator;

impl HtmlGenerator {
    pub fn to_html(&self, node: ftd::node::Node) -> String {
        let style = format!(
            "style=\"{}\"",
            self.style_to_html(&node, /*self.visible*/ true)
        );
        let classes = self.class_to_html(&node);
        let attrs = self.attrs_to_html(&node);

        let body = match node.text.as_ref() {
            Some(v) => v.to_string(),
            None => node
                .children
                .into_iter()
                .map(|v| self.to_html(v))
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
        let mut styles = node.style.to_owned();
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
            .map(|(k, v)| format!("{}={}", *k, quote(v))) // TODO: escape needed?
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
