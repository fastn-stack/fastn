pub struct OperationData {
    pub rects_buffer: wgpu::Buffer,
    // vertices: Vec<Triangle>,
    // textures: Vec<Image>,
    // glyphs: Vec<Glyph>,
}

impl OperationData {
    pub fn new(size: winit::dpi::PhysicalSize<u32>, document: &mut fastn_runtime::Document, device: &wgpu::Device,) -> OperationData {
          let (_ctrl, ops) = document.initial_layout(size.width, size.height);
        OperationData::draw(ops, device)
    }

    pub fn draw(ops: Vec<fastn_runtime::Operation>, device: &wgpu::Device,) -> OperationData {
        let mut rects = fastn_runtime::wgpu::rect::RectData::new();
        for op in ops.into_iter() {
            match op {
                fastn_runtime::Operation::DrawRectangle(rect) => {
                    rects.add(rect);
                }
            }
        }

        OperationData {
            rects_buffer: rects.upload(device),
        }
    }
}
