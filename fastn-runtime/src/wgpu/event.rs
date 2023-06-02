impl From<winit::event::Event<'_, ()>> for fastn_runtime::ExternalEvent {
    fn from(evt: winit::event::Event<()>) -> Self {
        dbg!(&evt);

        match evt {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    fastn_runtime::ExternalEvent::CursorMoved {
                        x: position.x,
                        y: position.y,
                    }
                }
                _ => fastn_runtime::ExternalEvent::NoOp,
            },
            _ => fastn_runtime::ExternalEvent::NoOp,
        }
    }
}
