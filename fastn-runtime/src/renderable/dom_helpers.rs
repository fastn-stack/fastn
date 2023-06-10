impl fastn_runtime::Dom {
    pub fn nodes_under_mouse(
        &self,
        key: fastn_runtime::NodeKey,
        pos_x: f64,
        pos_y: f64,
    ) -> Vec<fastn_runtime::NodeKey> {
        let node = self.nodes.get(key).unwrap();
        let mut node_keys = vec![];
        match node {
            fastn_runtime::Element::Container(c) => {
                // no need to draw a rectangle if there is no color or border
                if let Some(o) = c.operation(&self.taffy) {
                    if o.has_position(pos_x, pos_y) {
                        node_keys.push(key);
                        for child in self.children.get(key).unwrap() {
                            node_keys.extend(self.nodes_under_mouse(*child, pos_x, pos_y));
                        }
                    }
                }
            }
            fastn_runtime::Element::Text(_t) => todo!(),
            fastn_runtime::Element::Image(_i) => todo!(),
        }
        node_keys
    }
}
