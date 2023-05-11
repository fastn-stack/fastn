pub struct Document {
    pub taffy: taffy::Taffy,
    pub nodes: slotmap::SlotMap<fastn_surface::NodeKey, fastn_surface::Element>,
    pub root: fastn_surface::NodeKey,
    pub width: u32,
    pub height: u32,
    // variables, bindings
}

impl Document {
    // initial_html() -> server side HTML
    // hydrate() -> client side
    // event_with_target() -> Vec<DomMutation>

    // if not wasm
    pub fn initial_layout(
        &mut self,
        width: u32,
        height: u32,
    ) -> (fastn_surface::ControlFlow, Vec<fastn_surface::Operation>) {
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
        self.width = width;
        self.height = height;
        dbg!(self.taffy.layout(taffy_root).unwrap());
        (fastn_surface::ControlFlow::Wait, vec![])
    }

    pub async fn event(
        &mut self,
        _e: fastn_surface::Event,
    ) -> (fastn_surface::ControlFlow, Vec<fastn_surface::Operation>) {
        (fastn_surface::ControlFlow::Wait, vec![])
    }
}

impl Default for Document {
    fn default() -> Document {
        let mut nodes = slotmap::SlotMap::with_key();
        let mut taffy = taffy::Taffy::new();
        let root = nodes.insert(fastn_surface::Container::outer_column(&mut taffy));
        Document {
            root,
            taffy,
            nodes,
            width: 0,
            height: 0,
        }
    }
}
