impl From<winit::event::Event<'_, ()>> for fastn_runtime::Event {
    fn from(_value: winit::event::Event<()>) -> Self {
        todo!()
    }
}
