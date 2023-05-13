pub struct RectData {
    pub rects: Vec<fastn_runtime::operation::Rectangle>,
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
