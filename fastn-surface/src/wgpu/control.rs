impl From<winit::event_loop::ControlFlow> for fastn_surface::ControlFlow {
    fn from(value: winit::event_loop::ControlFlow) -> Self {
        match value {
            winit::event_loop::ControlFlow::Poll => fastn_surface::ControlFlow::Poll,
            winit::event_loop::ControlFlow::ExitWithCode(_) => fastn_surface::ControlFlow::Exit,
            winit::event_loop::ControlFlow::Wait => fastn_surface::ControlFlow::Wait,
            winit::event_loop::ControlFlow::WaitUntil(value) => {
                fastn_surface::ControlFlow::WaitUntil(value)
            }
        }
    }
}
