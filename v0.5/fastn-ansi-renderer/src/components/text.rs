use crate::{ComponentRenderer, LayoutConstraints, ComponentLayout, Canvas, Position, Rect};

/// Text component ASCII renderer
#[derive(Debug, Clone)]
pub struct TextRenderer {
    pub text: String,
    pub color: Option<String>,
    pub role: Option<String>,
    pub border_width: Option<usize>,
    pub padding: Option<usize>,
    pub width: Option<usize>,
}

impl TextRenderer {
    pub fn new(text: String) -> Self {
        Self {
            text,
            color: None,
            role: None,
            border_width: None,
            padding: None,
            width: None,
        }
    }

    pub fn with_border(mut self, width: usize) -> Self {
        self.border_width = Some(width);
        self
    }

    pub fn with_padding(mut self, padding: usize) -> Self {
        self.padding = Some(padding);
        self
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }
}

impl ComponentRenderer for TextRenderer {
    fn calculate_layout(&self, constraints: &LayoutConstraints) -> ComponentLayout {
        let text_width = self.text.chars().count();
        let text_height = if let Some(constrained_width) = self.width {
            // Calculate wrapped height
            self.calculate_wrapped_height(constrained_width)
        } else {
            1
        };

        let padding = self.padding.unwrap_or(0);
        let border = if self.border_width.is_some() { 2 } else { 0 }; // left + right border

        let content_width = self.width.unwrap_or(text_width);
        let total_width = content_width + (padding * 2) + border;
        let total_height = text_height + (padding * 2) + border;

        ComponentLayout {
            width: total_width.min(constraints.max_width.unwrap_or(usize::MAX)),
            height: total_height.min(constraints.max_height.unwrap_or(usize::MAX)),
            content_width,
            content_height: text_height,
        }
    }

    fn render(&self, canvas: &mut Canvas, layout: &ComponentLayout) {
        // Draw border if specified
        if self.border_width.is_some() {
            let rect = Rect {
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

        // Draw text with wrapping if width is constrained
        let wrap_width = self.width.map(|w| w);
        canvas.draw_text(
            Position { x: text_x, y: text_y },
            &self.text,
            wrap_width,
        );
    }
}

impl TextRenderer {
    fn calculate_wrapped_height(&self, width: usize) -> usize {
        if width == 0 {
            return 1;
        }

        let words: Vec<&str> = self.text.split_whitespace().collect();
        let mut current_line_length = 0;
        let mut lines = 1;

        for word in words {
            let word_length = word.len();
            
            if current_line_length + word_length + 1 <= width {
                // Word fits on current line
                if current_line_length > 0 {
                    current_line_length += 1; // space
                }
                current_line_length += word_length;
            } else {
                // Word goes to next line
                lines += 1;
                current_line_length = word_length;
            }
        }

        lines
    }
}