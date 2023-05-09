pub fn render(mut w: fastn_surface::Document) {
    env_logger::init();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();
    let window_size = window.inner_size();

    w.layout(window_size.width, window_size.height);

    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            winit::event::WindowEvent::CloseRequested
            | winit::event::WindowEvent::KeyboardInput {
                input:
                winit::event::KeyboardInput {
                    state: winit::event::ElementState::Pressed,
                    virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                    ..
                },
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,
            _ => {}
        },
        _ => {}
    })
}
