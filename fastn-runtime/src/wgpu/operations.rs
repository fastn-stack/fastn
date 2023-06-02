pub struct OperationData {
    pub rect_data: fastn_runtime::wgpu::rectangles::RectData,
    // vertices: Vec<Triangle>,
    // textures: Vec<Image>,
    // glyphs: Vec<Glyph>,
}

impl OperationData {
    pub fn new(
        size: winit::dpi::PhysicalSize<u32>,
        document: &mut fastn_runtime::Document,
        w: &fastn_runtime::wgpu::boilerplate::Wgpu,
    ) -> OperationData {
        let (_ctrl, ops) = document.compute_layout(size.width, size.height);
        let mut rects = vec![];
        for op in ops.into_iter() {
            match op {
                fastn_runtime::Operation::DrawRectangle(rect) => {
                    rects.push(dbg!(rect));
                }
            }
        }
        OperationData {
            rect_data: fastn_runtime::wgpu::rectangles::RectData::new(size, rects, w),
        }
    }
}
