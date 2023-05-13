pub struct RectData {
    pub rects: Vec<fastn_runtime::operation::Rectangle>,
}

#[allow(dead_code)]
struct Triangle {
    a: [f32; 2],
    b: [f32; 2],
    c: [f32; 2],
}

impl RectData {
    pub fn new() -> Self {
        Self { rects: Vec::new() }
    }

    pub fn add(&mut self, rect: fastn_runtime::operation::Rectangle) {
        self.rects.push(rect);
    }

    pub fn upload(self, device: &wgpu::Device) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.rects),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }
}
