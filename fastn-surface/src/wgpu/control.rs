impl From<fastn_surface::ControlFlow> for  winit::event_loop::ControlFlow {
    fn from(value: fastn_surface::ControlFlow ) -> Self {
        match value {
            fastn_surface::ControlFlow::Exit => winit::event_loop::ControlFlow::ExitWithCode(0),
            fastn_surface::ControlFlow::Wait => winit::event_loop::ControlFlow::Wait,
            fastn_surface::ControlFlow::WaitUntil(value) => winit::event_loop::ControlFlow::WaitUntil(value),
        }
    }
}
