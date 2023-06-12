pub struct Dom {}

impl fastn_runtime::DomT for Dom {
    fn create_kernel(&mut self, parent: fastn_runtime::NodeKey, _k: fastn_runtime::ElementKind) -> fastn_runtime::NodeKey {
        todo!()
    }

    fn add_child(&mut self, parent_key: fastn_runtime::NodeKey, child_key: fastn_runtime::NodeKey) {
        todo!()
    }
}