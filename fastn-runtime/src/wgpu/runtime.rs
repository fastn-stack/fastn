pub async fn render_document(document: fastn_runtime::Document) {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(window, document).await;

    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window.id() => match event {
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
            winit::event::WindowEvent::Resized(physical_size) => {
                state.wgpu.resize(*physical_size);
                state.window.request_redraw();
            }
            winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                state.wgpu.resize(**new_inner_size);
                state.window.request_redraw();
            }
            _ => {
                // dbg!(event);
            }
        },
        winit::event::Event::RedrawRequested(window_id) if window_id == state.window.id() => {
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.wgpu.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        winit::event::Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            // state.window().request_redraw();
        }
        _ => {
            // by default the event_loop keeps calling this function, we can set it to ::Wait
            // to wait till the next event occurs.
            // dbg!(event, &control_flow);
            *control_flow = winit::event_loop::ControlFlow::Wait;
        }
    })
}

struct State {
    #[allow(dead_code)]
    document: fastn_runtime::Document,
    size: winit::dpi::PhysicalSize<u32>,
    wgpu: fastn_runtime::wgpu::boilerplate::Wgpu,
    window: winit::window::Window,
    #[allow(dead_code)]
    operation_data: fastn_runtime::wgpu::operations::OperationData,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl State {
    fn render(&self) -> Result<(), wgpu::SurfaceError> {
        Ok(())
    }

    // Creating some of the wgpu types requires async code
    async fn new(window: winit::window::Window, mut document: fastn_runtime::Document) -> Self {
        let size = window.inner_size();
        let wgpu = fastn_runtime::wgpu::boilerplate::Wgpu::new(&window, &size).await;

        let operation_data =
            fastn_runtime::wgpu::operations::OperationData::new(size, &mut document, &wgpu);

        let render_pipeline = fastn_runtime::wgpu::rectangles::render_pipeline(&wgpu);

        State {
            size,
            window,
            wgpu,
            document,
            operation_data,
            render_pipeline,
        }
    }
}
