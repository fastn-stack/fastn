pub struct RectData {
    pub rects: Vec<fastn_runtime::operation::Rectangle>,
}

impl RectData {
    pub fn new() -> Self {
        Self { rects: Vec::new() }
    }

    pub fn upload(&self) -> wgpu::Buffer {
        todo!()
    }
}