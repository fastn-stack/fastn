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

impl fastn_runtime::element::Container {
    pub fn operation(&self, taffy: &taffy::Taffy) -> Option<Operation> {
        let layout = taffy.layout(self.taffy).unwrap();

        match self.style.background_color {
            None => None,
            Some(c) => Some(Operation::DrawRectangle(Rectangle {
                top: layout.location.x as u32,
                left: layout.location.y as u32,
                width: layout.size.width as u32,
                height: layout.size.height as u32,
                color: c,
            })),
        }
    }
}