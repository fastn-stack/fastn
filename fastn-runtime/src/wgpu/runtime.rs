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
    wgpu: fastn_runtime::wgpu::Wgpu,
    window: winit::window::Window,
    #[allow(dead_code)]
    operation_data: fastn_runtime::wgpu::OperationData,
}

impl State {
    fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.wgpu.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.wgpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.operation_data.rect_data.pipeline);
            render_pass.set_vertex_buffer(0, self.operation_data.rect_data.buffer.slice(..));
            render_pass.draw(0..3, 0..1);
        }

        self.wgpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    // Creating some of the wgpu types requires async code
    async fn new(window: winit::window::Window, mut document: fastn_runtime::Document) -> Self {
        let size = window.inner_size();
        let wgpu = fastn_runtime::wgpu::boilerplate::Wgpu::new(&window, &size).await;

        let operation_data = fastn_runtime::wgpu::OperationData::new(size, &mut document, &wgpu);

        State {
            size,
            window,
            wgpu,
            document,
            operation_data,
        }
    }
}
