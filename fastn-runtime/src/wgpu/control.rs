impl From<fastn_runtime::ControlFlow> for winit::event_loop::ControlFlow {
    fn from(value: fastn_runtime::ControlFlow) -> Self {
        match value {
            fastn_runtime::ControlFlow::Exit => winit::event_loop::ControlFlow::ExitWithCode(0),
            fastn_runtime::ControlFlow::WaitForEvent => winit::event_loop::ControlFlow::Wait,
            fastn_runtime::ControlFlow::WaitForEventTill(value) => {
                winit::event_loop::ControlFlow::WaitUntil(value)
            }
        }
    }
}
