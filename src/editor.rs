use std::io::{Read, Write};

const TAB_STOP: usize = 8;
struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        crossterm::terminal::disable_raw_mode().expect("Could not disable raw mode");
        Output::clear_screen().expect("Error");
    }
}

pub(crate) fn editor(content: &str) -> fpm::Result<()> {
    editor_(content).map_err(|e| fpm::Error::UsageError {
        message: e.to_string(),
    })
}

struct CursorController {
    cursor_x: usize,
    cursor_y: usize,
    screen_columns: usize,
    screen_rows: usize,
    row_offset: usize,
    column_offset: usize,
    render_x: usize,
}

impl CursorController {
    fn new(win_size: (usize, usize)) -> CursorController {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            screen_columns: win_size.0,
            screen_rows: win_size.1,
            row_offset: 0,
            column_offset: 0,
            render_x: 0,
        }
    }

    fn scroll(&mut self, editor_rows: &EditorRows) {
        self.render_x = 0;
        if self.cursor_y < editor_rows.number_of_rows() {
            self.render_x = self.get_render_x(editor_rows.get_editor_row(self.cursor_y))
        }

        self.row_offset = std::cmp::min(self.row_offset, self.cursor_y);
        if self.cursor_y >= self.row_offset + self.screen_rows {
            self.row_offset = self.cursor_y - self.screen_rows + 1;
        }

        self.column_offset = std::cmp::min(self.column_offset, self.render_x);
        if self.render_x >= self.column_offset + self.screen_columns {
            self.column_offset = self.render_x - self.screen_columns + 1;
        }
    }

    fn get_render_x(&self, row: &Row) -> usize {
        row.row_content[..self.cursor_x]
            .chars()
            .fold(0, |render_x, c| {
                if c == '\t' {
                    render_x + (TAB_STOP - 1) - (render_x % TAB_STOP) + 1
                } else {
                    render_x + 1
                }
            })
    }

    fn move_cursor(&mut self, direction: crossterm::event::KeyCode, editor_rows: &EditorRows) {
        let number_of_rows = editor_rows.number_of_rows();
        match direction {
            crossterm::event::KeyCode::Up => {
                self.cursor_y = self.cursor_y.saturating_sub(1);
            }
            crossterm::event::KeyCode::Left => {
                if self.cursor_x != 0 {
                    self.cursor_x -= 1;
                } else if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                    self.cursor_x = editor_rows.get_render(self.cursor_y).len();
                }
            }
            crossterm::event::KeyCode::Down => {
                if self.cursor_y < number_of_rows {
                    self.cursor_y += 1;
                }
            }
            crossterm::event::KeyCode::Right => {
                if self.cursor_y < number_of_rows {
                    match self
                        .cursor_x
                        .cmp(&editor_rows.get_render(self.cursor_y).len())
                    {
                        std::cmp::Ordering::Less => self.cursor_x += 1,
                        std::cmp::Ordering::Equal => {
                            self.cursor_y += 1;
                            self.cursor_x = 0
                        }
                        _ => {}
                    }
                }
            }
            crossterm::event::KeyCode::End => self.cursor_x = self.screen_columns - 1,
            crossterm::event::KeyCode::Home => self.cursor_x = 0,
            _ => unimplemented!(),
        }

        let row_len = if self.cursor_y < number_of_rows {
            editor_rows.get_render(self.cursor_y).len()
        } else {
            0
        };
        self.cursor_x = std::cmp::min(self.cursor_x, row_len);
    }
}

struct Reader;

impl Reader {
    fn read_key(&self) -> crossterm::Result<crossterm::event::KeyEvent> {
        loop {
            if crossterm::event::poll(std::time::Duration::from_millis(500))? {
                if let crossterm::event::Event::Key(event) = crossterm::event::read()? {
                    return Ok(event);
                }
            }
        }
    }
}

struct Row {
    row_content: Box<str>,
    render: String,
}

impl Row {
    fn new(row_content: Box<str>, render: String) -> Self {
        Self {
            row_content,
            render,
        }
    }
}

struct EditorRows {
    row_contents: Vec<Row>,
}

impl EditorRows {
    fn new(content: &str) -> Self {
        Self::from_argument(content)
    }

    fn from_argument(content: &str) -> Self {
        Self {
            row_contents: content
                .lines()
                .map(|it| {
                    let mut row = Row::new(it.into(), String::new());
                    Self::render_row(&mut row);
                    row
                })
                .collect(),
        }
    }

    fn render_row(row: &mut Row) {
        let mut index = 0;
        let capacity = row
            .row_content
            .chars()
            .fold(0, |acc, next| acc + if next == '\t' { TAB_STOP } else { 1 });
        let mut render = String::with_capacity(capacity);
        row.row_content.chars().for_each(|c| {
            index += 1;
            if c == '\t' {
                render.push(' ');
                while index % TAB_STOP != 0 {
                    render.push(' ');
                    index += 1
                }
            } else {
                render.push(c);
            }
        });
        row.render = render;
    }

    fn get_render(&self, at: usize) -> &String {
        &self.row_contents[at].render
    }

    fn get_editor_row(&self, at: usize) -> &Row {
        &self.row_contents[at]
    }

    fn number_of_rows(&self) -> usize {
        self.row_contents.len()
    }
}

struct Output {
    win_size: (usize, usize),
    editor_contents: EditorContents,
    cursor_controller: CursorController,
    editor_rows: EditorRows,
}

impl Output {
    fn new(content: &str) -> Self {
        let win_size = crossterm::terminal::size()
            .map(|(x, y)| (x as usize, y as usize))
            .unwrap();
        Self {
            win_size,
            editor_contents: EditorContents::new(),
            cursor_controller: CursorController::new(win_size),
            editor_rows: EditorRows::new(content),
        }
    }

    fn clear_screen() -> crossterm::Result<()> {
        crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )?;
        crossterm::execute!(std::io::stdout(), crossterm::cursor::MoveTo(0, 0))
    }

    fn draw_rows(&mut self) {
        let screen_rows = self.win_size.1;
        let screen_columns = self.win_size.0;
        for i in 0..screen_rows {
            let file_row = i + self.cursor_controller.row_offset;
            if file_row >= self.editor_rows.number_of_rows() {
                if self.editor_rows.number_of_rows() == 0 && i == screen_rows / 3 {
                    let mut welcome = "FPM Editor ---".to_string();
                    if welcome.len() > screen_columns {
                        welcome.truncate(screen_columns);
                    }
                    let mut padding = (screen_columns - welcome.len()) / 2;
                    if padding != 0 {
                        self.editor_contents.push('~');
                        padding -= 1;
                    }
                    (0..padding).for_each(|_| self.editor_contents.push(' '));
                    self.editor_contents.push_str(&welcome);
                } else {
                    self.editor_contents.push('~');
                }
            } else {
                let row = self.editor_rows.get_render(file_row);
                let column_offset = self.cursor_controller.column_offset;
                let len = std::cmp::min(row.len().saturating_sub(column_offset), screen_columns);
                let start = if len == 0 { 0 } else { column_offset };
                self.editor_contents.push_str(&row[start..start + len]);
            }
            crossterm::queue!(
                self.editor_contents,
                crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine)
            )
            .unwrap();
            if i < screen_rows - 1 {
                self.editor_contents.push_str("\r\n");
            }
        }
    }

    fn refresh_screen(&mut self) -> crossterm::Result<()> {
        self.cursor_controller.scroll(&self.editor_rows);
        crossterm::queue!(
            self.editor_contents,
            crossterm::cursor::Hide,
            crossterm::cursor::MoveTo(0, 0)
        )?;
        self.draw_rows();
        let cursor_x = self.cursor_controller.render_x - self.cursor_controller.column_offset;
        let cursor_y = self.cursor_controller.cursor_y - self.cursor_controller.row_offset;
        crossterm::queue!(
            self.editor_contents,
            crossterm::cursor::MoveTo(cursor_x as u16, cursor_y as u16),
            crossterm::cursor::Show
        )?;
        self.editor_contents.flush()
    }

    fn move_cursor(&mut self, direction: crossterm::event::KeyCode) {
        self.cursor_controller
            .move_cursor(direction, &self.editor_rows);
    }
}

struct EditorContents {
    content: String,
}

impl EditorContents {
    fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    fn push(&mut self, ch: char) {
        self.content.push(ch)
    }

    fn push_str(&mut self, string: &str) {
        self.content.push_str(string)
    }
}

impl std::io::Write for EditorContents {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.content.push_str(s);
                Ok(s.len())
            }
            Err(_) => Err(std::io::ErrorKind::WriteZero.into()),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let out = write!(std::io::stdout(), "{}", self.content);
        std::io::stdout().flush()?;
        self.content.clear();
        out
    }
}

struct Editor {
    reader: Reader,
    output: Output,
}

impl Editor {
    fn new(content: &str) -> Self {
        Self {
            reader: Reader,
            output: Output::new(content),
        }
    }

    fn process_keypress(&mut self) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Char('q'),
                modifiers: crossterm::event::KeyModifiers::CONTROL,
            } => return Ok(false),
            crossterm::event::KeyEvent {
                code:
                    direction @ (crossterm::event::KeyCode::Up
                    | crossterm::event::KeyCode::Down
                    | crossterm::event::KeyCode::Left
                    | crossterm::event::KeyCode::Right
                    | crossterm::event::KeyCode::Home
                    | crossterm::event::KeyCode::End),
                modifiers: crossterm::event::KeyModifiers::NONE,
            } => self.output.move_cursor(direction),
            crossterm::event::KeyEvent {
                code:
                    val @ (crossterm::event::KeyCode::PageUp | crossterm::event::KeyCode::PageDown),
                modifiers: crossterm::event::KeyModifiers::NONE,
            } => (0..self.output.win_size.1).for_each(|_| {
                self.output
                    .move_cursor(if matches!(val, crossterm::event::KeyCode::PageUp) {
                        crossterm::event::KeyCode::Up
                    } else {
                        crossterm::event::KeyCode::Down
                    });
            }),
            _ => {}
        }
        Ok(true)
    }

    fn run(&mut self) -> crossterm::Result<bool> {
        self.output.refresh_screen()?;
        self.process_keypress()
    }
}

fn editor_(content: &str) -> crossterm::Result<()> {
    let _clean_up = CleanUp;
    crossterm::terminal::enable_raw_mode()?;
    let mut editor = Editor::new(content);
    while editor.run()? {}
    Ok(())
}
