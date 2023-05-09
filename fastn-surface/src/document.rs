pub struct Document {
    pub taffy: taffy::Taffy,
    pub nodes: slotmap::SlotMap<fastn_surface::NodeKey, fastn_surface::Element>,
    pub root: fastn_surface::NodeKey,
}

impl Document {
    pub fn layout(&mut self, width: u32, height: u32) {
        let taffy_root = self.nodes[self.root].taffy();
        self.taffy
            .compute_layout(
                taffy_root,
                taffy::prelude::Size {
                    width: taffy::prelude::points(width as f32),
                    height: taffy::prelude::points(height as f32),
                },
            )
            .unwrap();
        dbg!(self.taffy.layout(taffy_root).unwrap());
    }
}

impl Default for Document {
    fn default() -> Document {
        let mut nodes = slotmap::SlotMap::with_key();
        let mut taffy = taffy::Taffy::new();
        let root = nodes.insert(fastn_surface::Container::outer_column(&mut taffy));
        Document { root, taffy, nodes }
    }
}
