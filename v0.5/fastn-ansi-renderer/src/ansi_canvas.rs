use ansi_term::{Color as AnsiTermColor, Style as AnsiStyle};

/// ANSI-capable canvas with color support
#[derive(Debug, Clone)]
pub struct AnsiCanvas {
    char_grid: Vec<Vec<char>>,
    fg_color_grid: Vec<Vec<AnsiColor>>,
    bg_color_grid: Vec<Vec<Option<AnsiColor>>>,
    width: usize,  // characters
    height: usize, // lines
}

impl AnsiCanvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            char_grid: vec![vec![' '; width]; height],
            fg_color_grid: vec![vec![AnsiColor::Default; width]; height],
            bg_color_grid: vec![vec![None; width]; height],
            width,
            height,
        }
    }

    /// Draw a border using Unicode box drawing characters
    pub fn draw_border(&mut self, rect: CharRect, style: BorderStyle, color: AnsiColor) {
        if rect.width < 2 || rect.height < 2 {
            return; // Too small for border
        }

        let chars = match style {
            BorderStyle::Single => BoxChars::single(),
            BorderStyle::Double => BoxChars::double(),
        };

        // Top border
        self.set_char_with_color(rect.x, rect.y, chars.top_left, color);
        for x in rect.x + 1..rect.x + rect.width - 1 {
            self.set_char_with_color(x, rect.y, chars.horizontal, color);
        }
        self.set_char_with_color(rect.x + rect.width - 1, rect.y, chars.top_right, color);

        // Bottom border
        let bottom_y = rect.y + rect.height - 1;
        self.set_char_with_color(rect.x, bottom_y, chars.bottom_left, color);
        for x in rect.x + 1..rect.x + rect.width - 1 {
            self.set_char_with_color(x, bottom_y, chars.horizontal, color);
        }
        self.set_char_with_color(rect.x + rect.width - 1, bottom_y, chars.bottom_right, color);

        // Side borders
        for y in rect.y + 1..rect.y + rect.height - 1 {
            self.set_char_with_color(rect.x, y, chars.vertical, color);
            self.set_char_with_color(rect.x + rect.width - 1, y, chars.vertical, color);
        }
    }

    /// Fill rectangle with background color
    pub fn draw_filled_rect(&mut self, rect: CharRect, bg_color: AnsiColor) {
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                self.set_bg_color(x, y, bg_color);
            }
        }
    }

    /// Draw text with colors
    pub fn draw_text(
        &mut self,
        pos: CharPos,
        text: &str,
        fg_color: AnsiColor,
        bg_color: Option<AnsiColor>,
    ) {
        for (i, ch) in text.chars().enumerate() {
            if pos.x + i < self.width && pos.y < self.height {
                self.set_char_with_colors(pos.x + i, pos.y, ch, fg_color, bg_color);
            }
        }
    }

    fn set_char_with_color(&mut self, x: usize, y: usize, ch: char, color: AnsiColor) {
        if x < self.width && y < self.height {
            self.char_grid[y][x] = ch;
            self.fg_color_grid[y][x] = color;
        }
    }

    fn set_char_with_colors(
        &mut self,
        x: usize,
        y: usize,
        ch: char,
        fg: AnsiColor,
        bg: Option<AnsiColor>,
    ) {
        if x < self.width && y < self.height {
            self.char_grid[y][x] = ch;
            self.fg_color_grid[y][x] = fg;
            self.bg_color_grid[y][x] = bg;
        }
    }

    fn set_bg_color(&mut self, x: usize, y: usize, color: AnsiColor) {
        if x < self.width && y < self.height {
            self.bg_color_grid[y][x] = Some(color);
        }
    }

    /// Convert canvas to ANSI string with color codes
    pub fn to_ansi_string(&self) -> String {
        let mut result = String::new();
        let mut current_fg = AnsiColor::Default;
        let mut current_bg: Option<AnsiColor> = None;

        for y in 0..self.height {
            for x in 0..self.width {
                let ch = self.char_grid[y][x];
                let fg = self.fg_color_grid[y][x];
                let bg = self.bg_color_grid[y][x];

                // Only add color codes when color changes
                if fg != current_fg || bg != current_bg {
                    if let Some(ansi_code) = self.get_ansi_style(fg, bg) {
                        result.push_str(&ansi_code);
                    }
                    current_fg = fg;
                    current_bg = bg;
                }

                result.push(ch);
            }

            // Reset colors at end of line and add newline
            result.push_str("\x1b[0m\n");
            current_fg = AnsiColor::Default;
            current_bg = None;
        }

        // Remove final newline and trim
        result.trim_end().to_string()
    }

    fn get_ansi_style(&self, fg: AnsiColor, bg: Option<AnsiColor>) -> Option<String> {
        match (fg, bg) {
            (AnsiColor::Default, None) => Some("\x1b[0m".to_string()),
            (fg, None) => Some(format!("\x1b[{}m", fg.to_ansi_code())),
            (AnsiColor::Default, Some(bg)) => Some(format!("\x1b[{}m", bg.to_ansi_bg_code())),
            (fg, Some(bg)) => Some(format!(
                "\x1b[{};{}m",
                fg.to_ansi_code(),
                bg.to_ansi_bg_code()
            )),
        }
    }
}

/// Character-based position
#[derive(Debug, Clone, Copy)]
pub struct CharPos {
    pub x: usize,
    pub y: usize,
}

/// Character-based rectangle
#[derive(Debug, Clone, Copy)]
pub struct CharRect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// ANSI color support
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnsiColor {
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl AnsiColor {
    fn to_ansi_code(&self) -> u8 {
        match self {
            AnsiColor::Default => 39,
            AnsiColor::Black => 30,
            AnsiColor::Red => 31,
            AnsiColor::Green => 32,
            AnsiColor::Yellow => 33,
            AnsiColor::Blue => 34,
            AnsiColor::Magenta => 35,
            AnsiColor::Cyan => 36,
            AnsiColor::White => 37,
            AnsiColor::BrightBlack => 90,
            AnsiColor::BrightRed => 91,
            AnsiColor::BrightGreen => 92,
            AnsiColor::BrightYellow => 93,
            AnsiColor::BrightBlue => 94,
            AnsiColor::BrightMagenta => 95,
            AnsiColor::BrightCyan => 96,
            AnsiColor::BrightWhite => 97,
        }
    }

    fn to_ansi_bg_code(&self) -> u8 {
        self.to_ansi_code() + 10 // Background codes are +10 from foreground
    }
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

/// Convert Taffy pixel coordinates to character coordinates
pub struct CoordinateConverter {
    char_width: f32,  // pixels per character
    line_height: f32, // pixels per line
}

impl CoordinateConverter {
    pub fn new() -> Self {
        Self {
            char_width: 8.0,   // Typical monospace character width
            line_height: 16.0, // Typical line height
        }
    }

    pub fn px_to_chars(&self, px: f32) -> usize {
        (px / self.char_width).round() as usize
    }

    pub fn px_to_lines(&self, px: f32) -> usize {
        (px / self.line_height).round() as usize
    }

    pub fn taffy_layout_to_char_rect(&self, layout: &taffy::Layout) -> CharRect {
        CharRect {
            x: self.px_to_chars(layout.location.x),
            y: self.px_to_lines(layout.location.y),
            width: self.px_to_chars(layout.size.width),
            height: self.px_to_lines(layout.size.height),
        }
    }

    /// Get content area inside padding and border
    pub fn get_content_area(&self, layout: &taffy::Layout) -> CharRect {
        let total_rect = self.taffy_layout_to_char_rect(layout);

        // Calculate padding in characters
        let padding_left = self.px_to_chars(layout.padding.left);
        let padding_top = self.px_to_lines(layout.padding.top);
        let padding_right = self.px_to_chars(layout.padding.right);
        let padding_bottom = self.px_to_lines(layout.padding.bottom);

        // For now, assume 1-character border if any border exists
        // TODO: Get actual border width from component style
        let border = 1; // Simplified for Week 2

        CharRect {
            x: total_rect.x + border + padding_left,
            y: total_rect.y + border + padding_top,
            width: total_rect
                .width
                .saturating_sub((border + padding_left) + (border + padding_right)),
            height: total_rect
                .height
                .saturating_sub((border + padding_top) + (border + padding_bottom)),
        }
    }
}

impl Default for CoordinateConverter {
    fn default() -> Self {
        Self::new()
    }
}
