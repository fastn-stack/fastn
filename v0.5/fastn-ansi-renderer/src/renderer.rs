use crate::{AsciiLayout, Canvas, ComponentLayout, LayoutConstraints};

/// Main ASCII renderer - parallel to HtmlData in fastn-runtime
pub struct AsciiData {
    layout: AsciiLayout,
}

impl AsciiData {
    /// Create AsciiData from compiled document (parallel to HtmlData::from_cd)  
    pub fn from_cd(_compiled_doc: &str) -> Self {
        // TODO: Implement document parsing and layout calculation
        Self {
            layout: AsciiLayout::empty(),
        }
    }

    /// Render to ASCII string (parallel to HtmlData::to_test_html)
    pub fn to_ascii(&self) -> String {
        let canvas = self.layout.render();
        canvas.to_string()
    }
}

/// Main ASCII renderer entry point
pub struct AsciiRenderer;

impl AsciiRenderer {
    /// Render a compiled document to ASCII
    pub fn render(_compiled_doc: &str) -> String {
        // TODO: Implement full rendering pipeline
        "<!-- ASCII Rendering Not Yet Implemented -->".to_string()
    }
}

/// Component-specific renderer trait
pub trait ComponentRenderer {
    fn calculate_layout(&self, constraints: &LayoutConstraints) -> ComponentLayout;
    fn render(&self, canvas: &mut Canvas, layout: &ComponentLayout);
}

/// Text component renderer
pub struct TextRenderer {
    pub text: String,
    pub color: Option<String>,
    pub role: Option<String>,
    pub border_width: Option<usize>,
    pub padding: Option<usize>,
}

impl ComponentRenderer for TextRenderer {
    fn calculate_layout(&self, constraints: &LayoutConstraints) -> ComponentLayout {
        let text_width = self.text.chars().count();
        let text_height = 1;

        let padding = self.padding.unwrap_or(0);
        let border = if self.border_width.is_some() { 2 } else { 0 }; // left + right border

        let total_width = text_width + (padding * 2) + border;
        let total_height = text_height + (padding * 2) + border;

        ComponentLayout {
            width: total_width.min(constraints.max_width.unwrap_or(usize::MAX)),
            height: total_height.min(constraints.max_height.unwrap_or(usize::MAX)),
            content_width: text_width,
            content_height: text_height,
        }
    }

    fn render(&self, canvas: &mut Canvas, layout: &ComponentLayout) {
        // Draw border if specified
        if let Some(_border_width) = self.border_width {
            let rect = crate::Rect {
                x: 0,
                y: 0,
                width: layout.width,
                height: layout.height,
            };
            canvas.draw_border(rect, crate::canvas::BorderStyle::Single);
        }

        // Calculate text position (accounting for border and padding)
        let border_offset = if self.border_width.is_some() { 1 } else { 0 };
        let padding = self.padding.unwrap_or(0);
        let text_x = border_offset + padding;
        let text_y = border_offset + padding;

        // Draw text
        canvas.draw_text(
            crate::Position { x: text_x, y: text_y },
            &self.text,
            None, // TODO: Handle wrapping
        );
    }
}