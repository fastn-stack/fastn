#[derive(Clone)]
pub enum Operation {
    DrawRectangle(Rectangle),
    // DrawImage(Image),
    // DrawGlyphCluster(Glyph),
}

#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    pub top: u32,
    pub left: u32,
    pub width: u32,
    pub height: u32,
    // if there is no color we do not have to draw the rectangle, unless border is present
    pub color: fastn_runtime::ColorValue,
    // pub scroll_x: u32,
    // border
    // fill
}
