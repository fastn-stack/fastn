pub struct RectData {
    pub count: u32,
    pub buffer: wgpu::Buffer,
    pub pipeline: wgpu::RenderPipeline,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

const ATTRIBS: [wgpu::VertexAttribute; 2] =
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

impl fastn_runtime::Rectangle {
    fn wasm_color(&self) -> [f32; 3] {
        [
            fastn_runtime::wgpu::color_u8_to_f32(20),
            fastn_runtime::wgpu::color_u8_to_f32(0),
            fastn_runtime::wgpu::color_u8_to_f32(0),
        ]
    }

    pub fn to_vertex(self, size: winit::dpi::PhysicalSize<u32>) -> Vec<Vertex> {
        /*
                                     Window                             (1, 1)
        ┌───────────┬────────────────────────────────────────────────▲──┐
        │           │                                                │  │
        │◄── left ─►│                                               top │
        │           │          Rectangle                             │  │
        Y           ┌──────────────────────────────────────┬--▲------▼--│
                    │ a                                  b │  │         │
        a           │                                      │  │         │
        x           │◄───────────── width ────────────────►│  │         │
        i           │                                      │height      │
        s           │                                      │  │         │
        │           │                                      │  │         │
        ▼           │ d                                 c  │  │         │
        │           └──────────────────────────────────────┴──▼──       │
        │                                                               │
        └────────────────────────── X axis ─► ──────────────────────────┘
        (-1, -1)

        Note: X goes from -1 to +1, left to right (in GPU coordinates).
              Y goes from +1 to -1, top to bottom.
              Center of the window is (0, 0).
        */

        let pixel_width = 2.0 / size.width as f32;
        let pixel_height = 2.0 / size.height as f32;

        // x goes from -1 to 1
        let a_x = self.left as f32 * pixel_width - 1.0;
        // y goes from 1 to -1
        let a_y = 1.0 - self.top as f32 * pixel_height;
        let b_x = (self.left + self.width) as f32 * pixel_width - 1.0;
        let d_y = 1.0 - (self.top + self.height) as f32 * pixel_height;

        let color = self.wasm_color();

        let a = Vertex {
            position: [a_x, a_y, 0.0],
            color,
        };
        let b = Vertex {
            position: [b_x, a_y, 0.0],
            color,
        };
        let c = Vertex {
            position: [b_x, d_y, 0.0],
            color,
        };
        let d = Vertex {
            position: [a_x, d_y, 0.0],
            color,
        };

        #[rustfmt::skip]
        let vertices = vec![ // vertices have to be counter clock wise
            a, d, b,
            b, d, c,
        ];

        vertices
    }
}

fn vertices(size: winit::dpi::PhysicalSize<u32>, v: Vec<fastn_runtime::Rectangle>) -> Vec<Vertex> {
    v.into_iter().flat_map(|r| r.to_vertex(size)).collect()
}

impl RectData {
    pub fn new(
        size: winit::dpi::PhysicalSize<u32>,
        v: Vec<fastn_runtime::operation::Rectangle>,
        w: &fastn_runtime::wgpu::boilerplate::Wgpu,
    ) -> Self {
        use wgpu::util::DeviceExt;
        let vertices = vertices(size, v);
        let buffer = w
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let pipeline = render_pipeline(w);
        RectData {
            buffer,
            pipeline,
            count: vertices.len() as u32,
        }
    }
}

fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &ATTRIBS,
    }
}

pub fn render_pipeline(wgpu: &fastn_runtime::wgpu::boilerplate::Wgpu) -> wgpu::RenderPipeline {
    let shader = wgpu
        .device
        .create_shader_module(wgpu::include_wgsl!("rectangles.wgsl"));
    let render_pipeline_layout =
        wgpu.device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
    wgpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
}
