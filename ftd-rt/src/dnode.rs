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
    pub visible: bool,
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

    pub fn style_to_html(&self, visible: bool) -> String {
        let mut styles = self.style.to_owned();
        if !visible {
            styles.insert("display".to_string(), "none".to_string());
        }
        styles
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

    pub fn to_html(&self, id: &str) -> String {
        let style = format!("style=\"{}\"", self.style_to_html(self.visible));
        let classes = format!("class=\"{}\"", self.class_to_html());

        let attrs = {
            let mut attr = self.attrs_to_html();
            let events = ftd_rt::event::group_by_js_event(&self.events);
            for (name, actions) in events {
                let event = format!("window.ftd.handle_event('{}', '{}')", id, actions);
                attr.push(' ');
                attr.push_str(&format!("{}={}", name, quote(&event)));
            }
            attr
        };

        if self.node == "img" {
            return format!("<img {attrs} {style}>", attrs = attrs, style = style);
        }
        let body = match self.text.as_ref() {
            Some(v) => v.to_string(),
            None => self
                .children
                .iter()
                .map(|v| v.to_html(id))
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
