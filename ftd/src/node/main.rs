#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Node {
    pub classes: Vec<String>,
    pub events: Vec<Event>,
    pub node: String,
    pub display: String,
    pub condition: Option<fastn_type::Expression>,
    pub attrs: ftd::Map<ftd::node::Value>,
    pub style: ftd::Map<ftd::node::Value>,
    pub children: Vec<Node>,
    pub text: ftd::node::Value,
    pub null: bool,
    pub data_id: String,
    pub line_number: usize,
    pub raw_data: Option<RawNodeData>,
    pub web_component: Option<WebComponentData>,
    pub device: Option<ftd::executor::Device>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct HTMLData {
    pub title: ftd::node::Value,
    pub og_title: ftd::node::Value,
    pub twitter_title: ftd::node::Value,
    pub description: ftd::node::Value,
    pub og_description: ftd::node::Value,
    pub twitter_description: ftd::node::Value,
    pub og_image: ftd::node::Value,
    pub twitter_image: ftd::node::Value,
    pub theme_color: ftd::node::Value,
}

impl ftd::executor::HTMLData {
    pub(crate) fn to_html_data(&self, doc_id: &str) -> HTMLData {
        HTMLData {
            title: ftd::node::Value::from_executor_value(
                self.title.value.to_owned(),
                self.title.to_owned(),
                None,
                doc_id,
            ),
            og_title: ftd::node::Value::from_executor_value(
                self.og_title.value.to_owned(),
                self.og_title.to_owned(),
                None,
                doc_id,
            ),
            twitter_title: ftd::node::Value::from_executor_value(
                self.twitter_title.value.to_owned(),
                self.twitter_title.to_owned(),
                None,
                doc_id,
            ),
            description: ftd::node::Value::from_executor_value(
                self.description.value.to_owned(),
                self.description.to_owned(),
                None,
                doc_id,
            ),
            og_description: ftd::node::Value::from_executor_value(
                self.og_description.value.to_owned(),
                self.og_description.to_owned(),
                None,
                doc_id,
            ),
            twitter_description: ftd::node::Value::from_executor_value(
                self.twitter_description.value.to_owned(),
                self.twitter_description.to_owned(),
                None,
                doc_id,
            ),
            og_image: ftd::node::Value::from_executor_value(
                self.og_image
                    .to_owned()
                    .map(|v| v.map(|v| v.src.value))
                    .value,
                self.og_image.to_owned(),
                Some(ftd::executor::RawImage::image_pattern()),
                doc_id,
            ),
            twitter_image: ftd::node::Value::from_executor_value(
                self.twitter_image
                    .to_owned()
                    .map(|v| v.map(|v| v.src.value))
                    .value,
                self.twitter_image.to_owned(),
                Some(ftd::executor::RawImage::image_pattern()),
                doc_id,
            ),
            theme_color: ftd::node::Value::from_executor_value(
                self.theme_color
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.theme_color.to_owned(),
                Some(ftd::executor::Color::color_pattern()),
                doc_id,
            ),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct RawNodeData {
    pub properties: Vec<(String, fastn_type::Property)>,
    pub iteration: Option<fastn_type::Loop>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct WebComponentData {
    pub properties: ftd::Map<fastn_type::PropertyValue>,
}

pub type Event = ftd::executor::Event;

impl Node {
    fn from_common(
        node: &str,
        display: &str,
        common: &ftd::executor::Common,
        doc_id: &str,
        anchor_ids: &mut Vec<String>,
    ) -> Node {
        Node {
            node: s(node),
            display: s(display),
            condition: common.condition.to_owned(),
            attrs: common.attrs(doc_id),
            style: common.style(doc_id, &mut [], anchor_ids),
            children: vec![],
            text: Default::default(),
            classes: common.classes(),
            null: common.is_dummy,
            events: common.event.clone(),
            data_id: common.data_id.clone(),
            line_number: common.line_number,
            raw_data: None,
            web_component: None,
            device: common.device.to_owned(),
        }
    }

    fn from_children(
        common: &ftd::executor::Common,
        children: &[ftd::executor::Element],
        doc_id: &str,
        display: &str,
        anchor_ids: &mut Vec<String>,
    ) -> Node {
        use itertools::Itertools;

        let attrs = common.attrs(doc_id);
        let mut classes = vec![];
        classes.extend(common.classes());

        let node = common.node();

        Node {
            node: s(node.as_str()),
            attrs,
            condition: common.condition.to_owned(),
            text: Default::default(),
            children: children
                .iter()
                .map(|v| v.to_node(doc_id, anchor_ids))
                .collect_vec(),
            style: common.style(doc_id, &mut classes, anchor_ids),
            classes,
            null: common.is_dummy,
            events: common.event.clone(),
            data_id: common.data_id.to_string(),
            line_number: common.line_number,
            display: s(display),
            raw_data: None,
            web_component: None,
            device: common.device.to_owned(),
        }
    }

    fn from_container(
        common: &ftd::executor::Common,
        container: &ftd::executor::Container,
        doc_id: &str,
        display: &str,
        anchor_ids: &mut Vec<String>,
        container_class: &str,
    ) -> Node {
        use itertools::Itertools;

        let mut attrs = common.attrs(doc_id);
        attrs.extend(container.attrs());
        let mut classes = container.add_class();
        classes.extend(common.classes());
        classes.push(container_class.to_string());

        let node = common.node();

        Node {
            node: s(node.as_str()),
            attrs,
            condition: common.condition.to_owned(),
            text: Default::default(),
            children: container
                .children
                .iter()
                .map(|v| v.to_node(doc_id, anchor_ids))
                .collect_vec(),
            style: {
                let mut style = common.style(doc_id, &mut classes, anchor_ids);
                style.extend(container.style(doc_id));
                style
            },
            classes,
            null: common.is_dummy,
            events: common.event.clone(),
            data_id: common.data_id.to_string(),
            line_number: common.line_number,
            display: s(display),
            raw_data: None,
            web_component: None,
            device: common.device.to_owned(),
        }
    }

    pub(crate) fn is_null(&self) -> bool {
        self.null
    }
}

impl ftd::executor::Element {
    pub fn to_node(&self, doc_id: &str, anchor_ids: &mut Vec<String>) -> Node {
        match self {
            ftd::executor::Element::Row(r) => r.to_node(doc_id, anchor_ids),
            ftd::executor::Element::Column(c) => c.to_node(doc_id, anchor_ids),
            ftd::executor::Element::Container(e) => e.to_node(doc_id, anchor_ids),
            ftd::executor::Element::Text(t) => t.to_node(doc_id, anchor_ids),
            ftd::executor::Element::Integer(t) => t.to_node(doc_id, anchor_ids),
            ftd::executor::Element::Decimal(t) => t.to_node(doc_id, anchor_ids),
            ftd::executor::Element::Boolean(t) => t.to_node(doc_id, anchor_ids),
            ftd::executor::Element::Image(i) => i.to_node(doc_id, anchor_ids),
            ftd::executor::Element::Code(c) => c.to_node(doc_id, anchor_ids),
            ftd::executor::Element::Iframe(i) => i.to_node(doc_id, anchor_ids),
            ftd::executor::Element::TextInput(i) => i.to_node(doc_id, anchor_ids),
            ftd::executor::Element::CheckBox(c) => c.to_node(doc_id, anchor_ids),
            ftd::executor::Element::Rive(r) => r.to_node(doc_id, anchor_ids),
            ftd::executor::Element::Null { line_number } => Node {
                classes: vec![],
                events: vec![],
                node: "".to_string(),
                display: "".to_string(),
                condition: None,
                attrs: Default::default(),
                style: Default::default(),
                children: vec![],
                text: Default::default(),
                null: true,
                data_id: "".to_string(),
                line_number: *line_number,
                raw_data: None,
                web_component: None,
                device: None,
            },
            ftd::executor::Element::RawElement(r) => r.to_node(doc_id, anchor_ids),
            ftd::executor::Element::IterativeElement(i) => i.to_node(doc_id, anchor_ids),
            ftd::executor::Element::WebComponent(w) => w.to_node(),
            ftd::executor::Element::Document(_) => unreachable!(),
        }
    }
}

impl ftd::executor::IterativeElement {
    pub fn to_node(&self, doc_id: &str, anchor_ids: &mut Vec<String>) -> Node {
        let mut node = self.element.clone().to_node(doc_id, anchor_ids);
        if let Some(raw_data) = &mut node.raw_data {
            raw_data.iteration = Some(self.iteration.clone());
        }
        node
    }
}

impl ftd::executor::WebComponent {
    pub fn to_node(&self) -> Node {
        let name = if let Some((_, name)) = self.name.split_once('#') {
            name.to_string()
        } else {
            self.name.to_string()
        };

        Node {
            node: name,
            display: s("unset"),
            null: false,
            line_number: self.line_number,
            raw_data: None,
            web_component: Some(WebComponentData {
                properties: self.properties.to_owned(),
            }),
            ..Default::default()
        }
    }
}

impl ftd::executor::RawElement {
    pub fn to_node(&self, doc_id: &str, anchor_ids: &mut Vec<String>) -> Node {
        Node {
            node: s(self.name.as_str()),
            display: s("flex"),
            condition: self.condition.to_owned(),
            attrs: Default::default(),
            style: Default::default(),
            children: self
                .children
                .iter()
                .map(|v| v.to_node(doc_id, anchor_ids))
                .collect(),
            text: Default::default(),
            classes: Default::default(),
            null: true,
            events: self.events.clone(),
            data_id: format!("{}_id", self.name),
            line_number: self.line_number,
            raw_data: Some(RawNodeData {
                properties: self.properties.clone(),
                iteration: None,
            }),
            web_component: None,
            device: None,
        }
    }
}

impl ftd::executor::Row {
    pub fn to_node(&self, doc_id: &str, anchor_ids: &mut Vec<String>) -> Node {
        use ftd::node::utils::CheckMap;

        let mut n = Node::from_container(
            &self.common,
            &self.container,
            doc_id,
            "flex",
            anchor_ids,
            "ft_row",
        );

        let align_content_value = ftd::node::Value::from_executor_value(
            self.container
                .align_content
                .to_owned()
                .map(|v| v.map(|a| a.to_css_justify_content(true)))
                .value,
            self.container.align_content.to_owned(),
            Some(ftd::executor::Alignment::justify_content_pattern(true)),
            doc_id,
        );

        n.style.check_and_insert(
            "justify-content",
            ftd::node::Value::from_executor_value(
                self.container
                    .spacing
                    .to_owned()
                    .map(|v| v.map(|v| v.to_justify_content_css_string()))
                    .value,
                self.container.spacing.to_owned(),
                Some(ftd::executor::Spacing::justify_content_pattern()),
                doc_id,
            ),
        );

        if let Some(jc) = n.style.get_mut("justify-content") {
            if let Some(old_value) = jc.value.as_ref() {
                if old_value.eq("unset") {
                    jc.value = align_content_value.value;
                }
            }
            jc.properties.extend(align_content_value.properties);
        } else {
            n.style
                .check_and_insert("justify-content", align_content_value);
        }

        n.style.check_and_insert(
            "align-items",
            ftd::node::Value::from_executor_value(
                self.container
                    .align_content
                    .to_owned()
                    .map(|v| v.map(|a| a.to_css_align_items(true)))
                    .value,
                self.container.align_content.to_owned(),
                Some(ftd::executor::Alignment::align_item_pattern(true)),
                doc_id,
            ),
        );
        n
    }
}

impl ftd::executor::Column {
    pub fn to_node(&self, doc_id: &str, anchor_ids: &mut Vec<String>) -> Node {
        use ftd::node::utils::CheckMap;

        let mut n = Node::from_container(
            &self.common,
            &self.container,
            doc_id,
            "flex",
            anchor_ids,
            "ft_column",
        );

        let align_content_value = ftd::node::Value::from_executor_value(
            self.container
                .align_content
                .to_owned()
                .map(|v| v.map(|a| a.to_css_justify_content(false)))
                .value,
            self.container.align_content.to_owned(),
            Some(ftd::executor::Alignment::justify_content_pattern(false)),
            doc_id,
        );

        n.style.check_and_insert(
            "justify-content",
            ftd::node::Value::from_executor_value(
                self.container
                    .spacing
                    .to_owned()
                    .map(|v| v.map(|v| v.to_justify_content_css_string()))
                    .value,
                self.container.spacing.to_owned(),
                Some(ftd::executor::Spacing::justify_content_pattern()),
                doc_id,
            ),
        );

        if let Some(jc) = n.style.get_mut("justify-content") {
            if let Some(old_value) = jc.value.as_ref() {
                if old_value.eq("unset") {
                    jc.value = align_content_value.value;
                }
            }
            jc.properties.extend(align_content_value.properties);
        } else {
            n.style
                .check_and_insert("justify-content", align_content_value);
        }

        n.style.check_and_insert(
            "align-items",
            ftd::node::Value::from_executor_value(
                self.container
                    .align_content
                    .to_owned()
                    .map(|v| v.map(|a| a.to_css_align_items(false)))
                    .value,
                self.container.align_content.to_owned(),
                Some(ftd::executor::Alignment::align_item_pattern(false)),
                doc_id,
            ),
        );
        n
    }
}

impl ftd::executor::ContainerElement {
    pub fn to_node(&self, doc_id: &str, anchor_ids: &mut Vec<String>) -> Node {
        let mut n = Node::from_children(
            &self.common,
            &self.children,
            doc_id,
            self.display
                .value
                .as_ref()
                .map_or("block", |d| d.to_css_str()),
            anchor_ids,
        );
        if !self.common.is_not_visible && self.display.value.is_none() {
            n.style
                .insert(s("display"), ftd::node::Value::from_string("block"));
        }

        n
    }
}

impl ftd::executor::Text {
    pub fn to_node(&self, doc_id: &str, anchor_ids: &mut Vec<String>) -> Node {
        use ftd::node::utils::CheckMap;

        let node = self.common.node();
        let mut n = Node::from_common(node.as_str(), "block", &self.common, doc_id, anchor_ids);

        if self.common.region.value.is_some() {
            n.attrs.insert_if_not_contains(
                "id",
                ftd::node::Value::from_string(slug::slugify(&self.text.value.original)),
            );
        }

        n.style.check_and_insert(
            "display",
            ftd::node::Value::from_executor_value(
                self.display
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_str().to_string()))
                    .value,
                self.display.to_owned(),
                None,
                doc_id,
            ),
        );

        n.style.check_and_insert(
            "text-indent",
            ftd::node::Value::from_executor_value(
                self.text_indent
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string(&self.common.device)))
                    .value,
                self.text_indent.to_owned(),
                None,
                doc_id,
            ),
        );

        n.style.check_and_insert(
            "font-style",
            ftd::node::Value::from_executor_value(
                self.style
                    .to_owned()
                    .map(|v| v.map(|v| v.font_style_string()))
                    .value,
                self.style.to_owned(),
                None,
                doc_id,
            ),
        );

        n.style.check_and_insert(
            "text-decoration",
            ftd::node::Value::from_executor_value(
                self.style
                    .to_owned()
                    .map(|v| v.map(|v| v.font_decoration_string()))
                    .value,
                self.style.to_owned(),
                None,
                doc_id,
            ),
        );

        n.style.check_and_insert(
            "font-weight",
            ftd::node::Value::from_executor_value(
                self.style
                    .to_owned()
                    .map(|v| v.map(|v| v.font_weight_string()))
                    .value,
                self.style.to_owned(),
                None,
                doc_id,
            ),
        );

        n.style.check_and_insert(
            "text-align",
            ftd::node::Value::from_executor_value(
                self.text_align
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.text_align.to_owned(),
                None,
                doc_id,
            ),
        );

        n.style.check_and_insert(
            "display",
            ftd::node::Value::from_executor_value_with_default(
                self.line_clamp
                    .to_owned()
                    .map(|v| v.map(|_| "-webkit-box".to_string()))
                    .value,
                self.line_clamp.to_owned(),
                Some(ftd::executor::LineClamp::display_pattern()),
                doc_id,
                Some(format!("\"{}\"", n.display)),
            ),
        );

        n.style.check_and_insert(
            "overflow",
            ftd::node::Value::from_executor_value(
                self.line_clamp
                    .to_owned()
                    .map(|v| v.map(|_| "hidden".to_string()))
                    .value,
                self.line_clamp.to_owned(),
                Some(ftd::executor::LineClamp::overflow_pattern()),
                doc_id,
            ),
        );

        n.style.check_and_insert(
            "-webkit-line-clamp",
            ftd::node::Value::from_executor_value(
                self.line_clamp
                    .to_owned()
                    .map(|v| v.map(|v| v.to_string()))
                    .value,
                self.line_clamp.to_owned(),
                None,
                doc_id,
            ),
        );

        n.style.check_and_insert(
            "-webkit-box-orient",
            ftd::node::Value::from_executor_value(
                self.line_clamp
                    .to_owned()
                    .map(|v| v.map(|_| "vertical".to_string()))
                    .value,
                self.line_clamp.to_owned(),
                Some(ftd::executor::LineClamp::webkit_box_orient_pattern()),
                doc_id,
            ),
        );

        n.classes.extend(self.common.add_class());
        n.classes.push("ft_md".to_string());
        n.text = ftd::node::Value::from_executor_value(
            Some(self.text.value.rendered.to_string()),
            self.text.clone(),
            None,
            doc_id,
        );
        n
    }
}

impl ftd::executor::Rive {
    pub fn to_node(&self, doc_id: &str, anchor_ids: &mut Vec<String>) -> Node {
        Node {
            node: s("canvas"),
            display: s("block"),
            condition: self.common.condition.to_owned(),
            attrs: self.attrs(doc_id),
            style: self.common.style(doc_id, &mut [], anchor_ids),
            children: vec![],
            text: Default::default(),
            classes: self.common.classes(),
            null: false,
            events: self.common.event.clone(),
            data_id: self.common.data_id.clone(),
            line_number: self.common.line_number,
            raw_data: None,
            web_component: None,
            device: self.common.device.to_owned(),
        }
    }

    fn attrs(&self, doc_id: &str) -> ftd::Map<ftd::node::Value> {
        use ftd::node::utils::CheckMap;

        let mut d: ftd::Map<ftd::node::Value> = self.common.attrs(doc_id);

        d.check_and_insert(
            "width",
            ftd::node::Value::from_executor_value(
                self.canvas_width
                    .to_owned()
                    .map(|v| v.map(|v| v.to_string()))
                    .value,
                self.canvas_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "height",
            ftd::node::Value::from_executor_value(
                self.canvas_height
                    .to_owned()
                    .map(|v| v.map(|v| v.to_string()))
                    .value,
                self.canvas_height.to_owned(),
                None,
                doc_id,
            ),
        );

        d
    }
}

impl ftd::executor::Code {
    pub fn to_node(&self, doc_id: &str, anchor_ids: &mut Vec<String>) -> Node {
        use ftd::node::utils::CheckMap;

        let node = self.common.node();
        let mut n = Node::from_common(node.as_str(), "block", &self.common, doc_id, anchor_ids);

        n.style.check_and_insert(
            "text-align",
            ftd::node::Value::from_executor_value(
                self.text_align
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.text_align.to_owned(),
                None,
                doc_id,
            ),
        );

        n.style.check_and_insert(
            "display",
            ftd::node::Value::from_executor_value_with_default(
                self.line_clamp
                    .to_owned()
                    .map(|v| v.map(|_| "-webkit-box".to_string()))
                    .value,
                self.line_clamp.to_owned(),
                Some(ftd::executor::LineClamp::display_pattern()),
                doc_id,
                Some(format!("\"{}\"", n.display)),
            ),
        );

        n.style.check_and_insert(
            "overflow",
            ftd::node::Value::from_executor_value(
                self.line_clamp
                    .to_owned()
                    .map(|v| v.map(|_| "hidden".to_string()))
                    .value,
                self.line_clamp.to_owned(),
                Some(ftd::executor::LineClamp::overflow_pattern()),
                doc_id,
            ),
        );

        n.style.check_and_insert(
            "-webkit-line-clamp",
            ftd::node::Value::from_executor_value(
                self.line_clamp
                    .to_owned()
                    .map(|v| v.map(|v| v.to_string()))
                    .value,
                self.line_clamp.to_owned(),
                None,
                doc_id,
            ),
        );

        n.style.check_and_insert(
            "-webkit-box-orient",
            ftd::node::Value::from_executor_value(
                self.line_clamp
                    .to_owned()
                    .map(|v| v.map(|_| "vertical".to_string()))
                    .value,
                self.line_clamp.to_owned(),
                Some(ftd::executor::LineClamp::webkit_box_orient_pattern()),
                doc_id,
            ),
        );

        n.classes.extend(self.common.add_class());
        n.classes.push("ft_md".to_string());
        n.text = ftd::node::Value::from_executor_value(
            Some(self.text.value.rendered.to_string()),
            self.text.clone(),
            None,
            doc_id,
        );
        n
    }
}

impl ftd::executor::Iframe {
    pub fn to_node(&self, doc_id: &str, anchor_ids: &mut Vec<String>) -> Node {
        use ftd::node::utils::CheckMap;

        let mut n = Node::from_common("iframe", "block", &self.common, doc_id, anchor_ids);

        n.attrs.check_and_insert(
            "src",
            ftd::node::Value::from_executor_value(
                self.src.to_owned().value,
                self.src.to_owned(),
                None,
                doc_id,
            ),
        );

        n.attrs.check_and_insert(
            "srcdoc",
            ftd::node::Value::from_executor_value(
                self.srcdoc.to_owned().value,
                self.srcdoc.to_owned(),
                None,
                doc_id,
            ),
        );

        n.attrs.check_and_insert(
            "allowfullscreen",
            ftd::node::Value::from_string("allowfullscreen"),
        );

        n.attrs.check_and_insert(
            "mozallowfullscreen",
            ftd::node::Value::from_string("mozallowfullscreen"),
        );

        n.attrs.check_and_insert(
            "msallowfullscreen",
            ftd::node::Value::from_string("msallowfullscreen"),
        );

        n.attrs.check_and_insert(
            "oallowfullscreen",
            ftd::node::Value::from_string("oallowfullscreen"),
        );

        n.attrs.check_and_insert(
            "webkitallowfullscreen",
            ftd::node::Value::from_string("webkitallowfullscreen"),
        );

        n.attrs.check_and_insert(
            "loading",
            ftd::node::Value::from_executor_value(
                Some(self.loading.to_owned().map(|v| v.to_css_string()).value),
                self.loading.to_owned(),
                None,
                doc_id,
            ),
        );

        n.classes.extend(self.common.add_class());
        n.classes.push("ft_md".to_string());
        n
    }
}

impl ftd::executor::TextInput {
    pub fn to_node(&self, doc_id: &str, anchor_ids: &mut Vec<String>) -> Node {
        use ftd::node::utils::CheckMap;

        let node = if self.multiline.value {
            "textarea"
        } else {
            "input"
        };

        let mut n = Node::from_common(node, "block", &self.common, doc_id, anchor_ids);

        n.attrs.check_and_insert(
            "placeholder",
            ftd::node::Value::from_executor_value(
                self.placeholder.to_owned().value,
                self.placeholder.to_owned(),
                None,
                doc_id,
            ),
        );

        n.attrs.check_and_insert(
            "disabled",
            ftd::node::Value::from_executor_value(
                self.enabled
                    .to_owned()
                    .map(|v| {
                        v.map(|b| {
                            if b {
                                s(ftd::interpreter::FTD_IGNORE_KEY)
                            } else {
                                s(ftd::interpreter::FTD_NO_VALUE)
                            }
                        })
                    })
                    .value,
                self.enabled.to_owned(),
                Some(ftd::executor::TextInput::enabled_pattern()),
                doc_id,
            ),
        );

        n.attrs.check_and_insert(
            "type",
            ftd::node::Value::from_executor_value(
                self.type_
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.type_.to_owned(),
                None,
                doc_id,
            ),
        );

        if self.multiline.value {
            n.text = ftd::node::Value::from_executor_value(
                self.value.to_owned().value,
                self.value.to_owned(),
                None,
                doc_id,
            );
        } else {
            n.attrs.check_and_insert(
                "value",
                ftd::node::Value::from_executor_value(
                    self.value.to_owned().value,
                    self.value.to_owned(),
                    None,
                    doc_id,
                ),
            );
        }

        n.attrs.check_and_insert(
            "data-dv",
            ftd::node::Value::from_executor_value(
                self.default_value.to_owned().value,
                self.default_value.to_owned(),
                None,
                doc_id,
            ),
        );

        n.classes.extend(self.common.add_class());
        n.classes.push("ft_md".to_string());
        n
    }
}

impl ftd::executor::CheckBox {
    pub fn to_node(&self, doc_id: &str, anchor_ids: &mut Vec<String>) -> Node {
        use ftd::node::utils::CheckMap;

        let node = "input";

        let mut n = Node::from_common(node, "block", &self.common, doc_id, anchor_ids);

        n.attrs
            .check_and_insert("type", ftd::node::Value::from_string(s("checkbox")));

        n.attrs.check_and_insert(
            "checked",
            ftd::node::Value::from_executor_value(
                self.checked
                    .to_owned()
                    .map(|v| {
                        v.map(|b| {
                            if b {
                                s(ftd::interpreter::FTD_NO_VALUE)
                            } else {
                                s(ftd::interpreter::FTD_IGNORE_KEY)
                            }
                        })
                    })
                    .value,
                self.checked.to_owned(),
                Some(ftd::executor::CheckBox::checked_pattern()),
                doc_id,
            ),
        );

        n.attrs.check_and_insert(
            "disabled",
            ftd::node::Value::from_executor_value(
                self.enabled
                    .to_owned()
                    .map(|v| {
                        v.map(|b| {
                            if b {
                                s(ftd::interpreter::FTD_IGNORE_KEY)
                            } else {
                                s(ftd::interpreter::FTD_NO_VALUE)
                            }
                        })
                    })
                    .value,
                self.enabled.to_owned(),
                Some(ftd::executor::CheckBox::enabled_pattern()),
                doc_id,
            ),
        );

        n.classes.extend(self.common.add_class());
        n.classes.push("ft_md".to_string());
        n
    }
}

impl ftd::executor::Image {
    pub fn to_node(&self, doc_id: &str, anchor_ids: &mut Vec<String>) -> Node {
        return if self.common.link.value.is_some() {
            let mut n = Node::from_common("a", "block", &self.common, doc_id, anchor_ids);
            n.attrs.insert(
                s("data-id"),
                ftd::node::Value::from_string(format!("{}:parent", self.common.data_id).as_str()),
            );

            let img = update_img(self, doc_id, anchor_ids);
            n.children.push(img);
            n
        } else {
            update_img(self, doc_id, anchor_ids)
        };

        fn update_img(
            image: &ftd::executor::Image,
            doc_id: &str,
            anchor_ids: &mut Vec<String>,
        ) -> Node {
            let mut n = Node::from_common("img", "block", &image.common, doc_id, anchor_ids);
            n.classes.extend(image.common.add_class());
            n.attrs.insert(
                s("src"),
                ftd::node::Value::from_executor_value(
                    Some(image.src.value.light.value.to_string()),
                    image.src.to_owned(),
                    None,
                    doc_id,
                ),
            );
            n.style.insert(
                s("object-fit"),
                ftd::node::Value::from_executor_value(
                    image
                        .fit
                        .to_owned()
                        .value
                        .as_ref()
                        .map(|v| v.to_css_string()),
                    image.fit.to_owned(),
                    None,
                    doc_id,
                ),
            );
            n.attrs.insert(
                s("alt"),
                ftd::node::Value::from_executor_value(
                    image.alt.to_owned().value,
                    image.alt.to_owned(),
                    None,
                    doc_id,
                ),
            );
            n
        }
    }
}

impl ftd::executor::Common {
    fn classes(&self) -> Vec<String> {
        let mut classes = self.classes.to_owned().value;
        classes.push("ft_common".to_string());
        classes
    }

    fn attrs(&self, doc_id: &str) -> ftd::Map<ftd::node::Value> {
        use ftd::node::utils::CheckMap;

        let mut d: ftd::Map<ftd::node::Value> = Default::default();

        d.check_and_insert(
            "id",
            ftd::node::Value::from_executor_value(
                self.id.value.to_owned(),
                self.id.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "data-id",
            ftd::node::Value::from_string(self.data_id.as_str()),
        );

        d.check_and_insert(
            "class",
            ftd::node::Value::from_executor_value(
                Some(self.classes.to_owned().value.join(", ")),
                self.classes.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "href",
            ftd::node::Value::from_executor_value(
                self.link.value.as_ref().map(ToString::to_string),
                self.link.to_owned(),
                None,
                doc_id,
            ),
        );

        if self.open_in_new_tab.value.is_some() && self.open_in_new_tab.value.unwrap() {
            d.check_and_insert(
                "target",
                ftd::node::Value::from_executor_value(
                    Some(ftd::node::utils::escape("_blank")),
                    self.open_in_new_tab.to_owned(),
                    Some((s("if ({0}) {\"_blank\"} else {null}"), true)),
                    doc_id,
                ),
            );
        }

        d
    }

    fn style(
        &self,
        doc_id: &str,
        _classes: &mut [String],
        anchor_ids: &mut Vec<String>,
    ) -> ftd::Map<ftd::node::Value> {
        use ftd::node::utils::CheckMap;

        let mut d: ftd::Map<ftd::node::Value> = Default::default();

        if let Some(id) = self.id.value.as_ref() {
            if anchor_ids.contains(id) {
                d.check_and_insert("position", ftd::node::Value::from_string("relative"));
            }
        }

        if let Some(ftd::executor::Anchor::Id(anchor_id)) = self.anchor.value.as_ref() {
            anchor_ids.push(anchor_id.clone());
            d.check_and_insert("position", ftd::node::Value::from_string("absolute"));
        }

        if ftd::node::utils::has_click_event(self.event.as_slice()) {
            d.check_and_insert("cursor", ftd::node::Value::from_string("pointer"));
        }

        if self.is_not_visible {
            d.check_and_insert("display", ftd::node::Value::from_string("none"));
        }

        d.check_and_insert(
            "z-index",
            ftd::node::Value::from_executor_value(
                self.z_index.value.as_ref().map(|v| v.to_string()),
                self.z_index.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "opacity",
            ftd::node::Value::from_executor_value(
                self.opacity.value.as_ref().map(|v| v.to_string()),
                self.opacity.to_owned(),
                None,
                doc_id,
            ),
        );

        if self.sticky.value.is_some() {
            // When sticky is used, setting top = 0px  and left = 0px
            d.check_and_insert(
                "position",
                ftd::node::Value::from_executor_value(
                    Some(s("sticky")),
                    self.sticky.to_owned(),
                    Some((s("if ({0}) {\"sticky\"} else {\"static\"}"), true)),
                    doc_id,
                ),
            );
            if self.top.value.is_none()
                && self.bottom.value.is_none()
                && self.left.value.is_none()
                && self.right.value.is_none()
            {
                d.check_and_insert(
                    "top",
                    ftd::node::Value::from_executor_value_with_default(
                        Some(s("0px")),
                        self.sticky.to_owned(),
                        Some((s("if ({0}) {\"0px\"}"), true)),
                        doc_id,
                        self.top
                            .value
                            .as_ref()
                            .map(|v| v.to_css_string(&self.device)),
                    ),
                );
                d.check_and_insert(
                    "left",
                    ftd::node::Value::from_executor_value_with_default(
                        Some(s("0px")),
                        self.sticky.to_owned(),
                        Some((s("if ({0}) {\"0px\"}"), true)),
                        doc_id,
                        self.top
                            .value
                            .as_ref()
                            .map(|v| v.to_css_string(&self.device)),
                    ),
                );
            }
        }

        d.check_and_insert(
            "box-shadow",
            ftd::node::Value::from_executor_value(
                self.shadow
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.shadow.to_owned(),
                Some(ftd::executor::Shadow::box_shadow_pattern()),
                doc_id,
            ),
        );

        d.check_and_insert(
            "top",
            ftd::node::Value::from_executor_value(
                self.top
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.top.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "bottom",
            ftd::node::Value::from_executor_value(
                self.bottom
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.bottom.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "left",
            ftd::node::Value::from_executor_value(
                self.left
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.left.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "right",
            ftd::node::Value::from_executor_value(
                self.right
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.right.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "width",
            ftd::node::Value::from_executor_value(
                self.width
                    .value
                    .to_owned()
                    .map(|v| v.to_css_string(&self.device)),
                self.width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "align-self",
            ftd::node::Value::from_executor_value(
                self.align_self
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.align_self.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "resize",
            ftd::node::Value::from_executor_value(
                self.resize
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.resize.to_owned(),
                None,
                doc_id,
            ),
        );

        // html and css name only
        d.check_and_insert(
            "overflow",
            ftd::node::Value::from_executor_value(
                self.overflow
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.overflow.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "overflow",
            ftd::node::Value::from_executor_value(
                self.resize
                    .to_owned()
                    .map(|v| v.map(|_| "auto".to_string()))
                    .value,
                self.resize.to_owned(),
                None,
                doc_id,
            ),
        );

        // html and css name only
        d.check_and_insert(
            "overflow-x",
            ftd::node::Value::from_executor_value(
                self.overflow_x
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.overflow_x.to_owned(),
                None,
                doc_id,
            ),
        );

        // html and css name only
        d.check_and_insert(
            "overflow-y",
            ftd::node::Value::from_executor_value(
                self.overflow_y
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.overflow_y.to_owned(),
                None,
                doc_id,
            ),
        );

        // todo: need to fix conditionals working with background
        d.check_and_insert(
            "background-image",
            ftd::node::Value::from_executor_value(
                self.background
                    .to_owned()
                    .map(|v| v.map(|v| v.to_image_src_css_string(&self.device)))
                    .value,
                self.background.to_owned(),
                Some(ftd::executor::Background::background_image_pattern()),
                doc_id,
            ),
        );

        d.check_and_insert(
            "background-repeat",
            ftd::node::Value::from_executor_value(
                self.background
                    .to_owned()
                    .map(|v| v.map(|v| v.to_image_repeat_css_string()))
                    .value,
                self.background.to_owned(),
                Some(ftd::executor::Background::background_repeat_pattern()),
                doc_id,
            ),
        );

        d.check_and_insert(
            "background-size",
            ftd::node::Value::from_executor_value(
                self.background
                    .to_owned()
                    .map(|v| v.map(|v| v.to_image_size_css_string(&self.device)))
                    .value,
                self.background.to_owned(),
                Some(ftd::executor::Background::background_size_pattern()),
                doc_id,
            ),
        );

        d.check_and_insert(
            "background-position",
            ftd::node::Value::from_executor_value(
                self.background
                    .to_owned()
                    .map(|v| v.map(|v| v.to_image_position_css_string(&self.device)))
                    .value,
                self.background.to_owned(),
                Some(ftd::executor::Background::background_position_pattern()),
                doc_id,
            ),
        );

        d.check_and_insert(
            "background-color",
            ftd::node::Value::from_executor_value(
                self.background
                    .to_owned()
                    .map(|v| v.map(|v| v.to_solid_css_string()))
                    .value,
                self.background.to_owned(),
                Some(ftd::executor::Background::background_color_pattern()),
                doc_id,
            ),
        );

        d.check_and_insert(
            "color",
            ftd::node::Value::from_executor_value(
                self.color
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.color.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-color",
            ftd::node::Value::from_executor_value(
                self.border_color
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.border_color.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "cursor",
            ftd::node::Value::from_executor_value(
                self.cursor
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.cursor.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "position",
            ftd::node::Value::from_executor_value(
                self.anchor
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.anchor.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "font-size",
            ftd::node::Value::from_executor_value(
                self.role
                    .to_owned()
                    .map(|v| v.and_then(|v| v.to_css_font_size(&self.device)))
                    .value,
                self.role.to_owned(),
                Some(ftd::executor::ResponsiveType::font_size_pattern()),
                doc_id,
            ),
        );

        d.check_and_insert(
            "line-height",
            ftd::node::Value::from_executor_value(
                self.role
                    .to_owned()
                    .map(|v| v.and_then(|v| v.to_css_line_height(&self.device)))
                    .value,
                self.role.to_owned(),
                Some(ftd::executor::ResponsiveType::line_height_pattern()),
                doc_id,
            ),
        );

        d.check_and_insert(
            "letter-spacing",
            ftd::node::Value::from_executor_value(
                self.role
                    .to_owned()
                    .map(|v| v.and_then(|v| v.to_css_letter_spacing(&self.device)))
                    .value,
                self.role.to_owned(),
                Some(ftd::executor::ResponsiveType::letter_spacing_pattern()),
                doc_id,
            ),
        );

        d.check_and_insert(
            "font-weight",
            ftd::node::Value::from_executor_value(
                self.role
                    .to_owned()
                    .map(|v| v.and_then(|v| v.to_css_weight(&self.device)))
                    .value,
                self.role.to_owned(),
                Some(ftd::executor::ResponsiveType::weight_pattern()),
                doc_id,
            ),
        );

        d.check_and_insert(
            "font-family",
            ftd::node::Value::from_executor_value(
                self.role
                    .to_owned()
                    .map(|v| v.and_then(|v| v.to_css_font_family()))
                    .value,
                self.role.to_owned(),
                Some(ftd::executor::ResponsiveType::font_family_pattern()),
                doc_id,
            ),
        );

        d.check_and_insert(
            "height",
            ftd::node::Value::from_executor_value(
                self.height
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.height.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding",
            ftd::node::Value::from_executor_value(
                self.padding
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.padding.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-left",
            ftd::node::Value::from_executor_value(
                self.padding_horizontal
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.padding_horizontal.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-right",
            ftd::node::Value::from_executor_value(
                self.padding_horizontal
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.padding_horizontal.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-top",
            ftd::node::Value::from_executor_value(
                self.padding_vertical
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.padding_vertical.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-bottom",
            ftd::node::Value::from_executor_value(
                self.padding_vertical
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.padding_vertical.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-top",
            ftd::node::Value::from_executor_value(
                self.padding_top
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.padding_top.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-bottom",
            ftd::node::Value::from_executor_value(
                self.padding_bottom
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.padding_bottom.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-left",
            ftd::node::Value::from_executor_value(
                self.padding_left
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.padding_left.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-right",
            ftd::node::Value::from_executor_value(
                self.padding_right
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.padding_right.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin",
            ftd::node::Value::from_executor_value(
                self.margin
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.margin.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-left",
            ftd::node::Value::from_executor_value(
                self.margin_horizontal
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.margin_horizontal.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-right",
            ftd::node::Value::from_executor_value(
                self.margin_horizontal
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.margin_horizontal.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-top",
            ftd::node::Value::from_executor_value(
                self.margin_vertical
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.margin_vertical.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-bottom",
            ftd::node::Value::from_executor_value(
                self.margin_vertical
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.margin_vertical.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-top",
            ftd::node::Value::from_executor_value(
                self.margin_top
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.margin_top.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-bottom",
            ftd::node::Value::from_executor_value(
                self.margin_bottom
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.margin_bottom.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-left",
            ftd::node::Value::from_executor_value(
                self.margin_left
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.margin_left.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-right",
            ftd::node::Value::from_executor_value(
                self.margin_right
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.margin_right.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "min-width",
            ftd::node::Value::from_executor_value(
                self.min_width
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.min_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "max-width",
            ftd::node::Value::from_executor_value(
                self.max_width
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.max_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "min-height",
            ftd::node::Value::from_executor_value(
                self.min_height
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.min_height.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "max-height",
            ftd::node::Value::from_executor_value(
                self.max_height
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.max_height.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-style",
            ftd::node::Value::from_executor_value(
                self.border_style.value.as_ref().map(|v| v.to_css_string()),
                self.border_style.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-left-style",
            ftd::node::Value::from_executor_value(
                self.border_style_horizontal
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.border_style_horizontal.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-left-style",
            ftd::node::Value::from_executor_value(
                self.border_style_left
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.border_style_left.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-right-style",
            ftd::node::Value::from_executor_value(
                self.border_style_right
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.border_style_right.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-right-style",
            ftd::node::Value::from_executor_value(
                self.border_style_horizontal
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.border_style_horizontal.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-top-style",
            ftd::node::Value::from_executor_value(
                self.border_style_top
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.border_style_top.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-top-style",
            ftd::node::Value::from_executor_value(
                self.border_style_vertical
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.border_style_vertical.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-bottom-style",
            ftd::node::Value::from_executor_value(
                self.border_style_bottom
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.border_style_bottom.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-bottom-style",
            ftd::node::Value::from_executor_value(
                self.border_style_vertical
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.border_style_vertical.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-bottom-width",
            ftd::node::Value::from_executor_value(
                self.border_width
                    .to_owned()
                    .map(|v| v.map(|l| l.to_css_string(&self.device)))
                    .value,
                self.border_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-top-width",
            ftd::node::Value::from_executor_value(
                self.border_width
                    .to_owned()
                    .map(|v| v.map(|l| l.to_css_string(&self.device)))
                    .value,
                self.border_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-left-width",
            ftd::node::Value::from_executor_value(
                self.border_width
                    .to_owned()
                    .map(|v| v.map(|l| l.to_css_string(&self.device)))
                    .value,
                self.border_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-right-width",
            ftd::node::Value::from_executor_value(
                self.border_width
                    .to_owned()
                    .map(|v| v.map(|l| l.to_css_string(&self.device)))
                    .value,
                self.border_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-bottom-width",
            ftd::node::Value::from_executor_value(
                self.border_bottom_width
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string(&self.device)))
                    .value,
                self.border_bottom_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-bottom-color",
            ftd::node::Value::from_executor_value(
                self.border_bottom_color
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.border_bottom_color.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-top-width",
            ftd::node::Value::from_executor_value(
                self.border_top_width
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string(&self.device)))
                    .value,
                self.border_top_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-top-color",
            ftd::node::Value::from_executor_value(
                self.border_top_color
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.border_top_color.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-left-width",
            ftd::node::Value::from_executor_value(
                self.border_left_width
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string(&self.device)))
                    .value,
                self.border_left_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-left-color",
            ftd::node::Value::from_executor_value(
                self.border_left_color
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.border_left_color.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-right-width",
            ftd::node::Value::from_executor_value(
                self.border_right_width
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string(&self.device)))
                    .value,
                self.border_right_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-right-color",
            ftd::node::Value::from_executor_value(
                self.border_right_color
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.border_right_color.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-radius",
            ftd::node::Value::from_executor_value(
                self.border_radius
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.border_radius.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-top-left-radius",
            ftd::node::Value::from_executor_value(
                self.border_top_left_radius
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.border_top_left_radius.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-top-right-radius",
            ftd::node::Value::from_executor_value(
                self.border_top_right_radius
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.border_top_right_radius.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-bottom-left-radius",
            ftd::node::Value::from_executor_value(
                self.border_bottom_left_radius
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.border_bottom_left_radius.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-bottom-right-radius",
            ftd::node::Value::from_executor_value(
                self.border_bottom_right_radius
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string(&self.device)),
                self.border_bottom_right_radius.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "white-space",
            ftd::node::Value::from_executor_value(
                self.white_space.value.as_ref().map(|v| v.to_css_string()),
                self.white_space.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "text-transform",
            ftd::node::Value::from_executor_value(
                self.text_transform
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.text_transform.to_owned(),
                None,
                doc_id,
            ),
        );

        d
    }

    fn add_class(&self) -> Vec<String> {
        // TODO: Implement add_class
        Default::default()
    }

    fn node(&self) -> String {
        if self.link.value.is_some() {
            s("a")
        } else if let Some(ref region) = self.region.value {
            region.to_css_string()
        } else {
            s("div")
        }
    }
}

impl ftd::executor::Container {
    fn attrs(&self) -> ftd::Map<ftd::node::Value> {
        // TODO: Implement attributes
        Default::default()
    }

    fn add_class(&self) -> Vec<String> {
        // TODO: Implement add_class
        Default::default()
    }

    fn style(&self, doc_id: &str) -> ftd::Map<ftd::node::Value> {
        use ftd::node::utils::CheckMap;

        let mut d: ftd::Map<ftd::node::Value> = Default::default();

        let count = ftd::node::utils::count_children_with_absolute_parent(self.children.as_slice());
        if count.gt(&0) {
            d.check_and_insert("position", ftd::node::Value::from_string("relative"));
        }

        d.check_and_insert(
            "gap",
            ftd::node::Value::from_executor_value(
                self.spacing
                    .value
                    .as_ref()
                    .map(|v| v.to_gap_css_string(&self.device)),
                self.spacing.to_owned(),
                Some(ftd::executor::Spacing::fixed_content_pattern()),
                doc_id,
            ),
        );

        d.check_and_insert(
            "flex-wrap",
            ftd::node::Value::from_executor_value(
                self.wrap
                    .value
                    .as_ref()
                    .map(|v| ftd::node::utils::wrap_to_css(*v)),
                self.wrap.to_owned(),
                Some((s("if ({0}) {\"wrap\"} else {\"nowrap\"}"), true)),
                doc_id,
            ),
        );

        d
    }
}

fn s(s: &str) -> String {
    s.to_string()
}
