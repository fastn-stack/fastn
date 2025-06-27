#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize, Default)]
pub struct DNode {
    pub classes: Vec<String>,
    pub node: String,
    pub attrs: ftd::Map<String>,
    pub style: ftd::Map<String>,
    pub children: Vec<DNode>,
    pub text: Option<String>,
    pub null: bool,
    pub visible: bool,
    pub events: Vec<ftd::Event>, // $on-click$: toggle foo | "click: toggle foo"
}

impl DNode {
    fn attrs_to_html(&self) -> String {
        self.attrs
            .iter()
            .map(|(k, v)| format!("{}={}", *k, quote(v)))
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
            .map(|(k, v)| format!("{}: {}", *k, ftd::html::escape(v))) // TODO: escape needed?
            .collect::<Vec<String>>()
            .join("; ")
    }

    pub fn class_to_html(&self) -> String {
        if self.classes.is_empty() {
            return "".to_string();
        }
        format!(
            "class=\"{}\"",
            self.classes
                .iter()
                .map(|k| k.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
    pub fn to_html(&self, id: &str) -> String {
        let style = format!("style=\"{}\"", self.style_to_html(self.visible));
        let classes = self.class_to_html();

        let attrs = {
            let mut attr = self.attrs_to_html();
            let events = ftd::ftd2021::event::group_by_js_event(&self.events);
            for (name, actions) in events {
                if name != "onclickoutside" && !name.starts_with("onglobalkey") {
                    let event = format!(
                        "window.ftd.handle_event(event, '{}', '{}', this)",
                        id,
                        actions.replace('\"', "&quot;")
                    );
                    attr.push(' ');
                    attr.push_str(&format!("{}={}", name, quote(&event)));
                }
            }
            attr
        };

        if self.node == "img" {
            return format!("<img {attrs} {style} {classes}>");
        }

        let body = match self.text.as_ref() {
            Some(v) => v.to_string(),
            None => self
                .children
                .iter()
                .map(|v| v.to_html(id))
                .collect::<Vec<String>>()
                .join(""),
        };

        // TODO: the generated tag should be indent properly. the `body` must be indented compared
        //       to open close tags
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
    format!("{i:?}")
}
