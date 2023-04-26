use dioxus_html::EventData;
use dioxus_native_core::{node::TextNode, prelude::*, real_dom::NodeImmutable, NodeId};
use rink::{render, Config, Driver};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

struct Document {}

impl Document {
    fn parse_ftd_document() -> ftd::node::NodeData {
        let doc = ftd::test_helper::ftd_v2_interpret_helper("foo", "-- ftd.text: hello world")
            .unwrap_or_else(|e| panic!("{:?}", e));
        let executor =
            ftd::executor::ExecuteDoc::from_interpreter(doc).unwrap_or_else(|e| panic!("{:?}", e));
        ftd::node::NodeData::from_rt(executor)
    }

    fn create(mut root: NodeMut, _node: ftd::node::NodeData) -> Self {
        let myself = Document {};

        let root_id = root.id();
        let rdom = root.real_dom_mut();

        let id = rdom
            .create_node(NodeType::Text(TextNode::new("count.to_string(), this is a long long long text, lets see if they know how to wrap text etc.".to_string())))
            .id();
        rdom.get_mut(root_id).unwrap().add_child(id);

        myself
    }
}

impl Driver for Document {
    fn update(&mut self, _: &Arc<RwLock<RealDom>>) {
        println!("Document.update()");
    }

    fn handle_event(
        &mut self,
        _: &Arc<RwLock<RealDom>>,
        _: NodeId,
        _: &str,
        _: Rc<EventData>,
        _: bool,
    ) {
        println!("Document.handle_event()");
    }

    fn poll_async(&mut self) -> std::pin::Pin<Box<dyn futures::Future<Output = ()> + '_>> {
        println!("Document.poll_async()");
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
