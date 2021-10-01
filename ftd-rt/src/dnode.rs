#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(serde::Serialize, PartialEq, Debug, Default, Clone)
)]
pub struct DNode {
    pub classes: Vec<String>,
    pub node: String,
    pub attrs: ftd_rt::Map,
    pub style: ftd_rt::Map,
    pub children: Vec<DNode>,
    pub text: Option<String>,
    pub null: bool,
    pub events: Vec<ftd_rt::Event>, // $event-click$: toggle foo | "click: toggle foo"
}

impl DNode {
    fn attrs_to_html(&self) -> String {
        self.attrs
            .iter()
            .map(|(k, v)| format!("{}={}", *k, quote(v))) // TODO: escape needed?
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn style_to_html(&self) -> String {
        self.style
            .iter()
            .map(|(k, v)| format!("{}: {}", *k, ftd_rt::html::escape(v))) // TODO: escape needed?
            .collect::<Vec<String>>()
            .join("; ")
    }

    pub fn class_to_html(&self) -> String {
        self.classes
            .iter()
            .map(|k| k.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn to_html(&self) -> String {
        let attrs = self.attrs_to_html();
        let style = format!("style=\"{}\"", self.style_to_html());
        let classes = format!("class=\"{}\"", self.class_to_html());

        if self.node == "img" {
            return format!("<img {attrs} {style}>", attrs = attrs, style = style);
        }
        let body = match self.text.as_ref() {
            Some(v) => v.to_string(),
            None => self
                .children
                .iter()
                .map(|v| v.to_html())
                .collect::<Vec<String>>()
                .join("\n"),
        };

        // TODO: indent things properly
        format!(
            "<{node} {attrs} {style} {classes}>{body}</{node}>",
            node = self.node.as_str(),
            attrs = attrs,
            style = style,
            classes = classes,
            body = body,
        )
    }
}

fn quote(i: &str) -> String {
    format!("{:?}", i)
}
