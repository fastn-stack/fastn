pub struct Window {
    pub taffy: taffy::Taffy,
    pub nodes: slotmap::SlotMap<fastn_surface::NodeKey, fastn_surface::Element>,
    pub root: fastn_surface::NodeKey,
}

impl Default for Window {
    fn default() -> Window {
        let mut nodes = slotmap::SlotMap::with_key();
        let mut taffy = taffy::Taffy::new();
        Window {
            root: nodes.insert(fastn_surface::Container::outer_column(&mut taffy)),
            taffy,
            nodes,
        }
    }
}
