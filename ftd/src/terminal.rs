use dioxus_html::EventData;
use dioxus_native_core::{
    node::OwnedAttributeDiscription, node::OwnedAttributeValue, node::TextNode, prelude::*,
    real_dom::NodeImmutable, NodeId,
};
use rink::{render, Config, Driver};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

struct Document {}

impl Document {
    fn parse_ftd_document() -> ftd::node::Node {
        let doc = ftd::test_helper::ftd_v2_interpret_helper("foo", ftd::terminal())
            .unwrap_or_else(|e| panic!("{:?}", e));
        let executor =
            ftd::executor::ExecuteDoc::from_interpreter(doc).unwrap_or_else(|e| panic!("{:?}", e));
        ftd::node::NodeData::from_rt(executor).node
    }

    fn create(mut root: NodeMut, node: ftd::node::Node) -> Self {
        let myself = Document {};

        let root_id = root.id();
        let rdom = root.real_dom_mut();

        let terminal_node = dbg!(node.to_terminal_node(rdom).id());

        rdom.get_mut(root_id).unwrap().add_child(terminal_node);

        myself
    }
}

impl ftd::node::Node {
    fn to_terminal_node(self, rdom: &mut RealDom) -> NodeMut {
        let mut attributes: rustc_hash::FxHashMap<OwnedAttributeDiscription, OwnedAttributeValue> =
            Default::default();

        for class in &self.classes {
            if class == "ft_column" {
                attributes.insert(("display", "style").into(), "flex".to_string().into());
                attributes.insert(("align-items", "style").into(), "start".to_string().into());
                attributes.insert(
                    ("justify-content", "style").into(),
                    "start".to_string().into(),
                );
                attributes.insert(
                    ("flex-direction", "style").into(),
                    "column".to_string().into(),
                );
            } else if class == "ft_row" {
                attributes.insert(("display", "style").into(), "flex".to_string().into());
                attributes.insert(("align-items", "style").into(), "start".to_string().into());
                attributes.insert(
                    ("justify-content", "style").into(),
                    "start".to_string().into(),
                );
                attributes.insert(("flex-direction", "style").into(), "row".to_string().into());
            }
            /*if class == "ft_common" {
                attributes.insert(("text-decoration", "style").into(), "none".to_string().into());
                attributes.insert(("box-sizing", "style").into(), "border-box".to_string().into());
                attributes.insert(("border-top-width", "style").into(), "0px".to_string().into());
                attributes.insert(("border-bottom-width", "style").into(), "0px".to_string().into());
                attributes.insert(("border-left-width", "style").into(), "0px".to_string().into());
                attributes.insert(("border-right-width", "style").into(), "0px".to_string().into());
                attributes.insert(("border-style", "style").into(), "solid".to_string().into());
                attributes.insert(("height", "style").into(), "auto".to_string().into());
                attributes.insert(("width", "style").into(), "auto".to_string().into());
            } */
        }

        for (k, v) in &self.attrs {
            if let Some(ref v) = v.value {
                attributes.insert(k.to_string().into(), v.to_string().into());
            }
        }

        for (k, v) in &self.style {
            if let Some(ref v) = v.value {
                attributes.insert((k.as_str(), "style").into(), v.to_string().into());
            }
        }

        let mut ele_id = vec![];
        for c in self.children {
            ele_id.push(c.to_terminal_node(rdom).id());
        }

        if let Some(text) = self.text.value {
            ele_id.push(rdom.create_node(NodeType::Text(TextNode::new(text))).id());
        }
        let mut nn = rdom.create_node(NodeType::Element(ElementNode {
            tag: self.node,
            attributes,
            ..Default::default()
        }));

        for id in ele_id {
            nn.add_child(id);
        }
        nn
    }
}

impl Driver for Document {
    fn update(&mut self, _: &Arc<RwLock<RealDom>>) {
        // println!("Document.update()");
    }

    fn handle_event(
        &mut self,
        _: &Arc<RwLock<RealDom>>,
        _: NodeId,
        _: &str,
        _: Rc<EventData>,
        _: bool,
    ) {
        // println!("Document.handle_event()");
    }

    fn poll_async(&mut self) -> std::pin::Pin<Box<dyn futures::Future<Output = ()> + '_>> {
        // println!("Document.poll_async()");
        // leaving this as is for now.
        Box::pin(async move { tokio::time::sleep(std::time::Duration::from_millis(1000)).await })
    }
}

pub fn run() {
    render(Config::new(), |rdom, _, _| {
        let mut rdom = rdom.write().unwrap();
        let root = rdom.root_id();
        Document::create(rdom.get_mut(root).unwrap(), Document::parse_ftd_document())
    })
    .unwrap();
}
