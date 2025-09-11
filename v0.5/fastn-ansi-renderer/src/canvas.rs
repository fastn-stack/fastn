/// Canvas for drawing ASCII art with Unicode box drawing characters
#[derive(Debug, Clone)]
pub struct Canvas {
    grid: Vec<Vec<char>>,
    width: usize,
    height: usize,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: vec![vec![' '; width]; height],
            width,
            height,
        }
    }

    /// Draw a border using Unicode box drawing characters
    pub fn draw_border(&mut self, rect: Rect, style: BorderStyle) {
        if rect.width < 2 || rect.height < 2 {
            return; // Too small for border
        }

        let chars = match style {
            BorderStyle::Single => BoxChars::single(),
            BorderStyle::Double => BoxChars::double(),
        };

        // Top border
        self.set_char(rect.x, rect.y, chars.top_left);
        for x in rect.x + 1..rect.x + rect.width - 1 {
            self.set_char(x, rect.y, chars.horizontal);
        }
        self.set_char(rect.x + rect.width - 1, rect.y, chars.top_right);

        // Bottom border
        let bottom_y = rect.y + rect.height - 1;
        self.set_char(rect.x, bottom_y, chars.bottom_left);
        for x in rect.x + 1..rect.x + rect.width - 1 {
            self.set_char(x, bottom_y, chars.horizontal);
        }
        self.set_char(rect.x + rect.width - 1, bottom_y, chars.bottom_right);

        // Side borders
        for y in rect.y + 1..rect.y + rect.height - 1 {
            self.set_char(rect.x, y, chars.vertical);
            self.set_char(rect.x + rect.width - 1, y, chars.vertical);
        }
    }

    /// Draw text at position with optional wrapping
    pub fn draw_text(&mut self, pos: Position, text: &str, wrap_width: Option<usize>) {
        match wrap_width {
            Some(width) => self.draw_wrapped_text(pos, text, width),
            None => self.draw_single_line_text(pos, text),
        }
    }

    fn draw_single_line_text(&mut self, pos: Position, text: &str) {
        for (i, ch) in text.chars().enumerate() {
            if pos.x + i < self.width && pos.y < self.height {
                self.set_char(pos.x + i, pos.y, ch);
            }
        }
    }

    fn draw_wrapped_text(&mut self, pos: Position, text: &str, width: usize) {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_line = String::new();
        let mut line_num = 0;

        for word in words {
            if current_line.len() + word.len() + 1 <= width {
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
            } else {
                // Draw current line and start new one
                self.draw_single_line_text(
                    Position {
                        x: pos.x,
                        y: pos.y + line_num,
                    },
                    &current_line,
                );
                current_line = word.to_string();
                line_num += 1;
            }
        }

        // Draw final line
        if !current_line.is_empty() {
            self.draw_single_line_text(
                Position {
                    x: pos.x,
                    y: pos.y + line_num,
                },
                &current_line,
            );
        }
    }

    fn set_char(&mut self, x: usize, y: usize, ch: char) {
        if x < self.width && y < self.height {
            self.grid[y][x] = ch;
        }
    }

    /// Convert canvas to string representation
    pub fn to_string(&self) -> String {
        self.grid
            .iter()
            .map(|row| row.iter().collect::<String>().trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n")
            .trim_end()
            .to_string()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum BorderStyle {
    Single,
    Double,
}

struct BoxChars {
    top_left: char,
    top_right: char,
    bottom_left: char,
    bottom_right: char,
    horizontal: char,
    vertical: char,
}

impl BoxChars {
    fn single() -> Self {
        Self {
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            horizontal: '─',
            vertical: '│',
        }
    }

    fn double() -> Self {
        Self {
            top_left: '╔',
            top_right: '╗',
            bottom_left: '╚',
            bottom_right: '╝',
            horizontal: '═',
            vertical: '║',
        }
    }
}
