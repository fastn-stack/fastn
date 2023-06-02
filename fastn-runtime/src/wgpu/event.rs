impl From<winit::event::Event<'_, ()>> for fastn_runtime::Event {
    fn from(evt: winit::event::Event<()>) -> Self {
        dbg!(&evt);

        match evt {
            winit::event::Event::WindowEvent {
                event, ..
            } => match event {
                winit::event::WindowEvent::CursorMoved {
                    position,
                    ..
                } => {
                    fastn_runtime::Event::CursorMoved {
                        x: position.x,
                        y: position.y,
                    }
                },
                _ => fastn_runtime::Event::NoOp
            }
            _ => fastn_runtime::Event::NoOp
        }

    }
}
