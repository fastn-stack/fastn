pub enum Operation {
    DrawRectangle(Rectangle),
    // DrawImage(Image),
    // DrawGlyphCluster(Glyph),
}

pub struct Rectangle {
    pub top: u32,
    pub left: u32,
    pub width: u32,
    pub height: u32,
    pub scroll_x: u32,
    // border
    // fill
}
