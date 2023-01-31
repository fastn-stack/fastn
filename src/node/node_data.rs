#![allow(dead_code)]

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct NodeData {
    pub name: String,
    pub node: ftd::node::Node,
    pub bag: ftd::Map<ftd::interpreter2::Thing>,
    pub aliases: ftd::Map<String>,
    pub dummy_node: ftd::Map<ftd::node::DummyNode>,
    pub raw_node: ftd::Map<ftd::node::RawNode>,
}

impl NodeData {
    #[tracing::instrument(skip_all)]
    pub fn from_rt(rt: ftd::executor::RT) -> NodeData {
        let node = rt.main.to_node("foo");
        let raw_node = dbg!(ftd::node::RawNode::from_element_constructors(
            rt.element_constructor,
            rt.name.as_str()
        ));
        let dummy_node = dbg!(ftd::node::DummyNode::from_dummy_instructions(
            rt.dummy_instructions,
            rt.name.as_str()
        ));

        NodeData {
            name: rt.name.to_string(),
            node,
            bag: rt.bag,
            aliases: rt.aliases,
            dummy_node,
            raw_node,
        }
    }
}
