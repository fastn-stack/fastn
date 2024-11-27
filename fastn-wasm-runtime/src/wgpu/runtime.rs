pub async fn render_document(document: fastn_runtime::Document) {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(window, document).await;

    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::WindowEvent {
            event: ref window_event,
            window_id,
        } if window_id == state.window.id() => match window_event {
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
                state.resize(*physical_size);
            }
            // display resolution changed (e.g. changing the resolution in the settings or switching
            // to monitor with different resolution)
            winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                state.resize(**new_inner_size);
            }
            _ => {
                *control_flow = state.handle_event(event);
            }
        },
        winit::event::Event::RedrawRequested(window_id) if window_id == state.window.id() => {
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
            *control_flow = winit::event_loop::ControlFlow::Wait;
        }
        winit::event::Event::RedrawEventsCleared => {
            *control_flow = winit::event_loop::ControlFlow::Wait;
        }
        winit::event::Event::NewEvents(_) => {
            *control_flow = winit::event_loop::ControlFlow::Wait;
        }
        winit::event::Event::MainEventsCleared => {
            // one or more events can come together, so we need to handle them all before we
            // re-render or re-compute the layout. winit::event::Event::MainEventsCleared is fired
            // after all events are handled.
            // https://docs.rs/winit/0.28.5/winit/event/enum.Event.html#variant.MainEventsCleared
            state.window.request_redraw();
        }
        _ => {
            *control_flow = state.handle_event(event);
        }
    })
}

struct State {
    document: fastn_runtime::Document,
    size: winit::dpi::PhysicalSize<u32>,
    wgpu: fastn_runtime::wgpu::Wgpu,
    window: winit::window::Window,
    operation_data: fastn_runtime::wgpu::OperationData,
}

impl State {
    pub fn handle_event(
        &mut self,
        event: winit::event::Event<()>,
    ) -> winit::event_loop::ControlFlow {
        let event: fastn_runtime::ExternalEvent = event.into();
        if event.is_nop() {
            return winit::event_loop::ControlFlow::Wait;
        }
        self.document.handle_event(event);
        self.operation_data =
            fastn_runtime::wgpu::OperationData::new(self.size, &mut self.document, &self.wgpu);
        self.window.request_redraw();
        winit::event_loop::ControlFlow::Wait
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.operation_data =
            fastn_runtime::wgpu::OperationData::new(new_size, &mut self.document, &self.wgpu);
        self.wgpu.resize(new_size);
    }

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
            render_pass.draw(0..self.operation_data.rect_data.count, 0..1);
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
