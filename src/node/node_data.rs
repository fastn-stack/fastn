#![allow(dead_code)]

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct NodeData {
    pub name: String,
    pub node: ftd::node::Node,
    pub bag: ftd::Map<ftd::interpreter2::Thing>,
    pub aliases: ftd::Map<String>,
}

impl NodeData {
    pub fn from_rt(rt: ftd::executor::RT) -> NodeData {
        let node = rt.main.to_node("foo");
        NodeData {
            name: rt.name.to_string(),
            node,
            bag: rt.bag,
            aliases: rt.aliases,
        }
    }
}
