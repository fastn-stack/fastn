pub struct Dom {
    pub taffy: taffy::Taffy,
    pub nodes: slotmap::SlotMap<fastn_runtime::NodeKey, fastn_runtime::Element>,
    pub root: fastn_runtime::NodeKey,
}

impl Dom {
    pub fn new() -> Dom {
        let mut nodes = slotmap::SlotMap::with_key();
        let mut taffy = taffy::Taffy::new();
        let root = nodes.insert(fastn_runtime::Container::outer_column(&mut taffy));

        Dom { taffy, nodes, root }
    }

    pub fn to_operations(&self) -> Vec<fastn_runtime::Operation> {
        vec![]
    }

    pub fn create_column(&mut self) -> fastn_runtime::NodeKey {
        self.nodes.insert(fastn_runtime::Container::outer_column(&mut self.taffy))
    }
}
