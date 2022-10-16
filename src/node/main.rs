#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Node {
    pub classes: Vec<String>,
    pub events: Vec<Event>,
    pub node: String,
    pub attrs: ftd::Map<ftd::node::Value>,
    pub style: ftd::Map<ftd::node::Value>,
    pub children: Vec<Node>,
    pub text: ftd::node::Value,
    pub null: bool,
}

pub type Event = ftd::executor::Event;

impl Node {
    fn from_common(node: &str, common: &ftd::executor::Common, doc_id: &str) -> Node {
        Node {
            node: s(node),
            attrs: common.attrs(),
            style: common.style(doc_id, &mut []),
            children: vec![],
            text: Default::default(),
            classes: vec![],
            null: common.is_dummy,
            events: common.event.clone(),
        }
    }

    fn from_container(
        common: &ftd::executor::Common,
        container: &ftd::executor::Container,
        doc_id: &str,
    ) -> Node {
        use itertools::Itertools;

        let mut attrs = common.attrs();
        attrs.extend(container.attrs());
        let mut classes = container.add_class();
        let mut style = common.style(doc_id, &mut classes);
        style.extend(container.style());

        let node = common.node();

        Node {
            node: s(node.as_str()),
            attrs,
            style,
            classes,
            text: Default::default(),
            children: container
                .children
                .iter()
                .map(|v| v.to_node(doc_id))
                .collect_vec(),
            null: common.is_dummy,
            events: common.event.clone(),
        }
    }
}

impl ftd::executor::Element {
    pub fn to_node(&self, doc_id: &str) -> Node {
        match self {
            ftd::executor::Element::Row(r) => r.to_node(doc_id),
            ftd::executor::Element::Column(c) => c.to_node(doc_id),
            ftd::executor::Element::Text(t) => t.to_node(doc_id),
        }
    }
}

impl ftd::executor::Row {
    pub fn to_node(&self, doc_id: &str) -> Node {
        let mut n = Node::from_container(&self.common, &self.container, doc_id);
        if !self.common.is_not_visible {
            n.style
                .insert(s("display"), ftd::node::Value::from_str("flex"));
        }
        n.style
            .insert(s("flex-direction"), ftd::node::Value::from_str("row"));

        n.style
            .insert(s("align-items"), ftd::node::Value::from_str("flex-start"));

        n.style.insert(
            s("justify-content"),
            ftd::node::Value::from_str("flex-start"),
        );

        n.children = {
            let mut children = vec![];
            for child in self.container.children.iter() {
                let child_node = child.to_node(doc_id);
                children.push(child_node);
            }
            children
        };
        n
    }
}

impl ftd::executor::Column {
    pub fn to_node(&self, doc_id: &str) -> Node {
        let mut n = Node::from_container(&self.common, &self.container, doc_id);
        if !self.common.is_not_visible {
            n.style
                .insert(s("display"), ftd::node::Value::from_str("flex"));
        }
        n.style
            .insert(s("flex-direction"), ftd::node::Value::from_str("column"));

        n.style
            .insert(s("align-items"), ftd::node::Value::from_str("flex-start"));

        n.style.insert(
            s("justify-content"),
            ftd::node::Value::from_str("flex-start"),
        );

        n.children = {
            let mut children = vec![];
            for child in self.container.children.iter() {
                let child_node = child.to_node(doc_id);
                children.push(child_node);
            }
            children
        };
        n
    }
}

impl ftd::executor::Text {
    pub fn to_node(&self, doc_id: &str) -> Node {
        let node = self.common.node();
        let mut n = Node::from_common(node.as_str(), &self.common, doc_id);
        n.classes.extend(self.common.add_class());
        n.classes.push("ft_md".to_string());
        n.text = ftd::node::Value::from_executor_value(
            Some(self.text.value.rendered.to_string()),
            self.text.clone(),
            None,
        );
        n
    }
}

impl ftd::executor::Common {
    fn attrs(&self) -> ftd::Map<ftd::node::Value> {
        // TODO: Implement attributes
        std::iter::IntoIterator::into_iter([(
            "data-id".to_string(),
            ftd::node::Value::from_str(self.data_id.as_str()),
        )])
        .collect()
    }

    fn style(&self, _doc_id: &str, _classes: &mut [String]) -> ftd::Map<ftd::node::Value> {
        let mut d: ftd::Map<ftd::node::Value> = Default::default();

        d.insert(s("text-decoration"), ftd::node::Value::from_str("none"));

        if self.is_not_visible {
            d.insert(s("display"), ftd::node::Value::from_str("none"));
        }

        d.insert(
            s("padding"),
            ftd::node::Value::from_executor_value(
                self.padding.value.map(|p| format!("{}px", p)),
                self.padding.to_owned(),
                Some(s("{}px")),
            ),
        );

        d
    }

    fn add_class(&self) -> Vec<String> {
        // TODO: Implement add_class
        Default::default()
    }

    fn node(&self) -> String {
        s("div")
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

    fn style(&self) -> ftd::Map<ftd::node::Value> {
        // TODO: Implement style
        Default::default()
    }
}

fn s(s: &str) -> String {
    s.to_string()
}
