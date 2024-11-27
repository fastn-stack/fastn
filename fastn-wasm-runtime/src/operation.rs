#[derive(Clone, Debug)]
pub enum Operation {
    DrawRectangle(Rectangle),
    // DrawImage(Image),
    // DrawGlyphCluster(Glyph),
}

impl Operation {
    pub(crate) fn has_position(&self, pos_x: f64, pos_y: f64) -> bool {
        match self {
            Operation::DrawRectangle(r) => r.has_position(pos_x, pos_y),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    pub top: u32,
    pub left: u32,
    pub width: u32,
    pub height: u32,
    // if there is no color we do not have to draw the rectangle, unless border is present
    // pub color: fastn_runtime::Color,
    // pub scroll_x: u32,
    // border
    // fill
}

impl Rectangle {
    pub(crate) fn has_position(&self, pos_x: f64, pos_y: f64) -> bool {
        let pos_x = pos_x as u32;
        let pos_y = pos_y as u32;
        pos_x >= self.top
            && pos_x <= self.top + self.height
            && pos_y >= self.left
            && pos_y <= self.left + self.width
    }
}

impl fastn_runtime::element::Container {
    pub fn operation(&self, taffy: &taffy::Taffy) -> Option<Operation> {
        let layout = taffy.layout(self.taffy_key).unwrap();

        Some(Operation::DrawRectangle(Rectangle {
            top: (layout.location.x as u32),
            left: (layout.location.y as u32),
            width: (layout.size.width as u32),
            height: (layout.size.height as u32),
            // color: c.light.to_owned(),
        }))
    }
}
