impl From<winit::event::Event<'_, ()>> for fastn_runtime::Event {
    fn from(evt: winit::event::Event<()>) -> Self {
        dbg!(evt);
        // match evt {
        //
        // }
        fastn_runtime::Event::NoOp
    }
}
