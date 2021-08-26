#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(serde::Serialize, PartialEq, Debug, Default, Clone)
)]
pub struct Node {
    pub condition: Option<ftd_rt::Condition>,
    pub classes: Vec<String>,
    pub node: String,
    pub attrs: ftd_rt::Map,
    pub style: ftd_rt::Map,
    pub children: Vec<Node>,
    pub children_style: ftd_rt::Map,
    pub text: Option<String>,
    pub null: bool,
}

impl Node {
    fn attrs_to_html(&self) -> String {
        self.attrs
            .iter()
            .map(|(k, v)| format!("{}={}", *k, quote(v))) // TODO: escape needed?
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn style_to_html(&self, style: &ftd_rt::Map) -> String {
        let mut s = self.style.clone();
        s.extend(style.clone());
        s.iter()
            .map(|(k, v)| format!("{}: {}", *k, escape(v))) // TODO: escape needed?
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

    pub fn fixed_children_style(&self, index: usize) -> ftd_rt::Map {
        if index == 0 {
            let mut list: ftd_rt::Map = Default::default();
            for (key, value) in self.children_style.iter() {
                if key == "margin-left" || key == "margin-top" {
                    continue;
                }
                list.insert(key.clone(), value.clone());
            }
            list
        } else {
            self.children_style.clone()
        }
    }

    pub fn is_visible(&self, data: &ftd_rt::Map) -> bool {
        if self.null {
            return false;
        }

        match self.condition {
            Some(ref v) => v.is_true(data),
            None => true,
        }
    }

    pub fn to_html(&self, style: &ftd_rt::Map, data: &ftd_rt::Map) -> String {
        if !self.is_visible(data) {
            return "".to_string();
        }

        let attrs = self.attrs_to_html();
        let style = format!("style=\"{}\"", self.style_to_html(style));
        let classes = format!("class=\"{}\"", self.class_to_html());

        if self.node == "img" {
            return format!("<img {attrs} {style}>", attrs = attrs, style = style);
        }

        let body = match self.text.as_ref() {
            Some(v) => v.to_string(),
            None => self
                .children
                .iter()
                .enumerate()
                .map(|(i, v)| v.to_html(&self.fixed_children_style(i), data))
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

impl ftd_rt::Element {
    pub fn to_node(&self) -> Node {
        match self {
            Self::Row(i) => (i.to_node()),
            Self::Text(i) => (i.to_node()),
            Self::Image(i) => (i.to_node()),
            Self::Column(i) => (i.to_node()),
            Self::IFrame(i) => (i.to_node()),
            Self::Input(i) => (i.to_node()),
            Self::Integer(i) => (i.to_node()),
            Self::Boolean(i) => (i.to_node()),
            Self::Decimal(i) => (i.to_node()),
            Self::Null => Node {
                condition: None,
                classes: vec![],
                node: "".to_string(),
                attrs: Default::default(),
                style: Default::default(),
                children: vec![],
                children_style: Default::default(),
                text: None,
                null: true,
            },
        }
    }

    // TODO: only when wasm feature is enabled
    pub fn to_dom(&self, _id: &str) {
        todo!()
    }
}

impl Node {
    fn from_common(node: &str, common: &ftd_rt::Common) -> Self {
        Node {
            condition: common.condition.clone(),
            node: s(node),
            attrs: common.attrs(),
            style: common.style(),
            children: vec![],
            children_style: common.children_style(),
            text: None,
            classes: common.add_class(),
            null: false,
        }
    }

    fn from_container(common: &ftd_rt::Common, container: &ftd_rt::Container) -> Self {
        let mut attrs = common.attrs();
        attrs.extend(container.attrs());
        let mut style = common.style();
        style.extend(container.style());
        let mut classes = common.add_class();
        classes.extend(container.add_class());

        let mut children_style = common.children_style();
        children_style.extend(container.children_style());

        Node {
            condition: common.condition.clone(),
            node: s("div"), // TODO: use better tags based on common.region
            attrs,
            style,
            classes,
            children_style,
            text: None,
            children: container.children.iter().map(|v| v.to_node()).collect(),
            null: false,
        }
    }
}

impl ftd_rt::Row {
    pub fn to_node(&self) -> Node {
        let mut n = Node::from_container(&self.common, &self.container);
        n.style.insert(s("display"), s("flex"));
        n.style.insert(s("flex-direction"), s("row"));
        if self.container.wrap {
            n.style.insert(s("flex-wrap"), s("wrap"));
        } else {
            n.style.insert(s("flex-wrap"), s("nowrap"));
        }
        for (key, value) in container_align(&self.container.align) {
            n.style.insert(s(key.as_str()), value);
        }
        n.style.insert(s("align-items"), s("flex-start"));

        n.style.insert(s("justify-content"), s("flex-start"));

        if let Some(p) = self.container.spacing {
            n.children_style
                .insert(s("margin-left"), format!("{}px", p));
        }

        n
    }
}

impl ftd_rt::Column {
    pub fn to_node(&self) -> Node {
        let mut n = Node::from_container(&self.common, &self.container);
        n.style.insert(s("display"), s("flex"));
        n.style.insert(s("flex-direction"), s("column"));
        if self.container.wrap {
            n.style.insert(s("flex-wrap"), s("wrap"));
        } else {
            n.style.insert(s("flex-wrap"), s("nowrap"));
        }
        for (key, value) in container_align(&self.container.align) {
            n.style.insert(s(key.as_str()), value);
        }
        n.style.insert(s("align-items"), s("flex-start"));

        n.style.insert(s("justify-content"), s("flex-start"));

        if let Some(p) = self.container.spacing {
            n.children_style.insert(s("margin-top"), format!("{}px", p));
        }

        n
    }
}

impl ftd_rt::Text {
    pub fn to_node(&self) -> Node {
        // TODO: proper tag based on self.common.region
        // TODO: if format is not markdown use pre
        let mut n = Node::from_common("div", &self.common);
        n.text = Some(self.text.rendered.clone());
        let (key, value) = text_align(&self.align);
        n.style.insert(s(key.as_str()), value);
        if let Some(p) = self.size {
            n.style.insert(s("font-size"), format!("{}px", p));
        }
        if let Some(p) = &self.line_height {
            n.style.insert(s("line-height"), format!("{}px", p));
        }
        // TODO: text styles
        n
    }
}

impl ftd_rt::Image {
    pub fn to_node(&self) -> Node {
        let mut n = Node::from_common("img", &self.common);
        if self.common.width == None {
            n.style.insert(s("width"), s("100%"));
        }
        n.attrs.insert(s("src"), escape(self.src.as_str()));
        n.attrs.insert(s("alt"), escape(self.description.as_str()));
        for (key, value) in container_align(&self.align) {
            n.style.insert(s(key.as_str()), value);
        }

        n
    }
}

impl ftd_rt::IFrame {
    pub fn to_node(&self) -> Node {
        let mut n = Node::from_common("iframe", &self.common);
        n.attrs.insert(s("src"), escape(self.src.as_str()));
        n
    }
}

impl ftd_rt::Input {
    pub fn to_node(&self) -> Node {
        let mut n = Node::from_common("input", &self.common);
        if let Some(ref p) = self.placeholder {
            n.attrs.insert(s("placeholder"), escape(p));
        }
        n
    }
}

impl ftd_rt::Common {
    fn add_class(&self) -> Vec<String> {
        let d: Vec<String> = vec![s("ft_md")];
        d
    }
    fn children_style(&self) -> ftd_rt::Map {
        let d: ftd_rt::Map = Default::default();
        d
    }

    fn style(&self) -> ftd_rt::Map {
        let mut d: ftd_rt::Map = Default::default();

        if let Some(p) = self.padding {
            d.insert(s("padding"), format!("{}px", p));
        }
        if let Some(p) = self.padding_left {
            d.insert(s("padding-left"), format!("{}px", p));
        }
        if let Some(p) = self.padding_right {
            d.insert(s("padding-right"), format!("{}px", p));
        }
        if let Some(p) = self.padding_top {
            d.insert(s("padding-top"), format!("{}px", p));
        }
        if let Some(p) = self.padding_bottom {
            d.insert(s("padding-bottom"), format!("{}px", p));
        }

        if let Some(p) = self.border_top_radius {
            d.insert(s("border-top-left-radius"), format!("{}px !important", p));
            d.insert(s("border-top-right-radius"), format!("{}px !important", p));
        }

        if let Some(p) = self.border_bottom_radius {
            d.insert(
                s("border-bottom-right-radius"),
                format!("{}px !important", p),
            );
            d.insert(
                s("border-bottom-left-radius"),
                format!("{}px !important", p),
            );
        }
        if let Some(p) = &self.width {
            let (key, value) = length(p, "width");
            d.insert(s(key.as_str()), value);
            // if p == &ftd_rt::Length::Fill {
            //     d.insert(s("flex-basis"), s("0"));
            //     d.insert(s("flex-grow"), s("1"));
            // }
        }
        if let Some(p) = &self.height {
            let (key, value) = length(p, "height");
            d.insert(s(key.as_str()), value);
        }
        if let Some(p) = self.border_left {
            d.insert(s("border-left-width"), format!("{}px !important", p));
        }
        if let Some(p) = self.border_right {
            d.insert(s("border-right-width"), format!("{}px !important", p));
        }
        if let Some(p) = self.border_top {
            d.insert(s("border-top-width"), format!("{}px !important", p));
        }
        if let Some(p) = self.border_bottom {
            d.insert(s("border-bottom-width"), format!("{}px !important", p));
        }
        if let Some(p) = self.margin_left {
            d.insert(s("margin-left"), format!("{}px", p));
        }
        if let Some(p) = self.margin_right {
            d.insert(s("margin-right"), format!("{}px", p));
        }
        if let Some(p) = self.margin_top {
            d.insert(s("margin-top"), format!("{}px", p));
        }
        if let Some(p) = self.margin_bottom {
            d.insert(s("margin-bottom"), format!("{}px", p));
        }
        if let Some(p) = &self.background_color {
            d.insert(s("background-color"), color(p));
        }
        if let Some(p) = &self.color {
            d.insert(s("color"), color(p));
        }
        if let Some(p) = &self.border_color {
            d.insert(s("border-color"), color(p));
        }
        if let Some(p) = &self.overflow_x {
            let (key, value) = overflow(p, "overflow-x");
            d.insert(s(key.as_str()), value);
        }
        if let Some(p) = &self.overflow_x {
            let (key, value) = overflow(p, "overflow-y");
            d.insert(s(key.as_str()), value);
        }
        d.insert(s("border-style"), s("solid"));
        d.insert(s("border-width"), format!("{}px", self.border_width));
        d.insert(s("border-radius"), format!("{}px", self.border_radius));
        d.insert(s("box-sizing"), s("border-box"));

        d
    }

    fn attrs(&self) -> ftd_rt::Map {
        let mut d: ftd_rt::Map = Default::default();
        if let Some(ref id) = self.id {
            d.insert(s("id"), escape(id));
        }
        if self.open_in_new_tab {
            d.insert(s("target"), escape("_blank"));
        }
        d
    }
}
impl ftd_rt::Container {
    fn style(&self) -> ftd_rt::Map {
        let d: ftd_rt::Map = Default::default();
        d
    }
    fn children_style(&self) -> ftd_rt::Map {
        let d: ftd_rt::Map = Default::default();
        d
    }

    fn attrs(&self) -> ftd_rt::Map {
        let d: ftd_rt::Map = Default::default();
        d
    }
    fn add_class(&self) -> Vec<String> {
        let d: Vec<String> = Default::default();
        d
    }
}

pub fn escape(s: &str) -> String {
    let s = s.replace('>', "\\u003E");
    let s = s.replace('<', "\\u003C");
    s.replace('&', "\\u0026")
}

fn quote(i: &str) -> String {
    format!("{:?}", i)
}

fn s(s: &str) -> String {
    s.to_string()
}

fn color(c: &ftd_rt::Color) -> String {
    let ftd_rt::Color { r, g, b, alpha } = c;
    format!("rgba({},{},{},{})", r, g, b, alpha)
}

fn length(l: &ftd_rt::Length, f: &str) -> (String, String) {
    let s = f.to_string();
    match l {
        ftd_rt::Length::Fill => (s, "100%".to_string()),
        ftd_rt::Length::Auto => (s, "auto".to_string()),
        ftd_rt::Length::Px { value } => (s, format!("{}px", value)),
        ftd_rt::Length::Portion { value } => ("flex-grow".to_string(), value.to_string()),
        ftd_rt::Length::Max { value } => (format!("max-{}", f), format!("{}px", value)),
        ftd_rt::Length::Min { value } => (format!("min-{}", f), format!("{}px", value)),
        ftd_rt::Length::Percent { value } => (s, format!("{}%", value)),
        _ => (s, "100%".to_string()),
        //        ftd_rt::Length::Shrink => (s, "width".to_string()),   TODO
    }
}

fn container_align(l: &ftd_rt::Align) -> Vec<(String, String)> {
    match l {
        ftd_rt::Align::Center => vec![
            ("align-self".to_string(), "center".to_string()),
            ("margin-bottom".to_string(), "auto".to_string()),
            ("margin-top".to_string(), "auto".to_string()),
        ],
        ftd_rt::Align::Top => vec![("align-self".to_string(), "center".to_string())],
        ftd_rt::Align::Left => vec![
            ("align-self".to_string(), "flex-start".to_string()),
            ("margin-bottom".to_string(), "auto".to_string()),
            ("margin-top".to_string(), "auto".to_string()),
        ],
        ftd_rt::Align::Right => vec![
            ("align-self".to_string(), "flex-end".to_string()),
            ("margin-bottom".to_string(), "auto".to_string()),
            ("margin-top".to_string(), "auto".to_string()),
            ("margin-left".to_string(), "auto".to_string()),
        ],
        ftd_rt::Align::Bottom => vec![
            ("align-self".to_string(), "center".to_string()),
            ("margin-bottom".to_string(), "0".to_string()),
            ("margin-top".to_string(), "auto".to_string()),
        ],
        ftd_rt::Align::TopLeft => vec![("align-self".to_string(), "flex-start".to_string())],
        ftd_rt::Align::TopRight => vec![
            ("align-self".to_string(), "flex-end".to_string()),
            ("margin-left".to_string(), "auto".to_string()),
        ],
        ftd_rt::Align::BottomLeft => vec![
            ("align-self".to_string(), "flex-start".to_string()),
            ("margin-bottom".to_string(), "0".to_string()),
            ("margin-top".to_string(), "auto".to_string()),
        ],
        ftd_rt::Align::BottomRight => vec![
            ("align-self".to_string(), "flex-end".to_string()),
            ("margin-bottom".to_string(), "0".to_string()),
            ("margin-top".to_string(), "auto".to_string()),
            ("margin-left".to_string(), "auto".to_string()),
        ],
    }
}

fn text_align(l: &ftd_rt::TextAlign) -> (String, String) {
    match l {
        ftd_rt::TextAlign::Center => ("text-align".to_string(), "center".to_string()),
        ftd_rt::TextAlign::Left => ("text-align".to_string(), "left".to_string()),
        ftd_rt::TextAlign::Right => ("text-align".to_string(), "right".to_string()),
        ftd_rt::TextAlign::Justify => ("text-align".to_string(), "center".to_string()),
    }
}

fn overflow(l: &ftd_rt::Overflow, f: &str) -> (String, String) {
    let s = f.to_string();
    match l {
        ftd_rt::Overflow::Auto => (s, "auto".to_string()),
        ftd_rt::Overflow::Hidden => (s, "hidden".to_string()),
        ftd_rt::Overflow::Scroll => (s, "scroll".to_string()),
        ftd_rt::Overflow::Visible => (s, "visible".to_string()),
    }
}
