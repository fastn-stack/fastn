pub struct OperationData {
    pub rects_buffer: wgpu::Buffer,
    // vertices: Vec<Triangle>,
    // textures: Vec<Image>,
    // glyphs: Vec<Glyph>,
}

impl OperationData {
    pub fn new(size: winit::dpi::PhysicalSize<u32>, document: &mut fastn_runtime::Document,) -> OperationData {
          let (_ctrl, ops) = document.initial_layout(size.width, size.height);
        OperationData::draw(ops)
    }

    pub fn draw(ops: Vec<fastn_runtime::Operation>) -> OperationData {
        let mut rects = fastn_runtime::wgpu::rect::RectData::new();
        for op in ops.into_iter() {
            match op {
                fastn_runtime::Operation::DrawRectangle(rect) => {
                    rects.rects.push(rect);
                }
            }
        }
        OperationData {
            rects_buffer: rects.upload(),
        }
    }
}