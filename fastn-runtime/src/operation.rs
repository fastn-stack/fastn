pub enum Operation {
    DrawRectangle(Rectangle),
    // DrawImage(Image),
    // DrawGlyphCluster(Glyph),
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
pub struct Rectangle {
    pub top: u32,
    pub left: u32,
    pub width: u32,
    pub height: u32,
    // pub scroll_x: u32,
    // border
    // fill
}
