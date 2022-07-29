use std::io::{Read, Write};

const TAB_STOP: usize = 8;
struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        crossterm::terminal::disable_raw_mode().expect("Could not disable raw mode");
        Output::clear_screen().expect("Error");
    }
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
            crossterm::event::KeyCode::End => {
                if self.cursor_y < number_of_rows {
                    self.cursor_x = editor_rows.get_render(self.cursor_y).len();
                }
            }
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

struct StatusMessage {
    message: Option<String>,
    set_time: Option<std::time::Instant>,
}

impl StatusMessage {
    fn new(initial_message: String) -> Self {
        Self {
            message: Some(initial_message),
            set_time: None,
        }
    }

    fn set_message(&mut self, message: String) {
        self.message = Some(message);
        self.set_time = Some(std::time::Instant::now())
    }

    fn message(&mut self) -> Option<&String> {
        self.set_time.and_then(move |time| {
            if time.elapsed() > std::time::Duration::from_secs(5) {
                self.message = None;
                self.set_time = None;
                None
            } else {
                Some(self.message.as_ref().unwrap())
            }
        })
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

#[derive(Default)]
struct Row {
    row_content: String,
    render: String,
}

impl Row {
    fn new(row_content: String, render: String) -> Self {
        Self {
            row_content,
            render,
        }
    }

    fn insert_char(&mut self, at: usize, ch: char) {
        self.row_content.insert(at, ch);
        EditorRows::render_row(self)
    }
}

struct EditorRows {
    row_contents: Vec<Row>,
    filename: Option<std::path::PathBuf>,
}

impl EditorRows {
    fn new(content: &str, filename: Option<std::path::PathBuf>) -> Self {
        Self::from_argument(content, filename)
    }

    fn from_argument(content: &str, filename: Option<std::path::PathBuf>) -> Self {
        Self {
            row_contents: content
                .lines()
                .map(|it| {
                    let mut row = Row::new(it.into(), String::new());
                    Self::render_row(&mut row);
                    row
                })
                .collect(),
            filename,
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

    fn insert_row(&mut self) {
        self.row_contents.push(Row::default());
    }

    fn get_editor_row_mut(&mut self, at: usize) -> &mut Row {
        &mut self.row_contents[at]
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

struct Output {
    win_size: (usize, usize),
    editor_contents: EditorContents,
    cursor_controller: CursorController,
    editor_rows: EditorRows,
    status_message: StatusMessage,
}

impl Output {
    fn new(content: &str, filename: Option<std::path::PathBuf>) -> Self {
        let win_size = crossterm::terminal::size()
            .map(|(x, y)| (x as usize, y as usize - 2)) // minus 2 for draw_status_bar and draw_message_bar
            .unwrap();
        Self {
            win_size,
            editor_contents: EditorContents::new(),
            cursor_controller: CursorController::new(win_size),
            editor_rows: EditorRows::new(content, filename),
            status_message: StatusMessage::new("HELP: Ctrl-Q = Quit".into()),
        }
    }

    fn draw_status_bar(&mut self) {
        self.editor_contents
            .push_str(&crossterm::style::Attribute::Reverse.to_string());
        let info = format!(
            "{} -- {} lines",
            self.editor_rows
                .filename
                .as_ref()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .unwrap_or("[No Name]"),
            self.editor_rows.number_of_rows()
        );
        let info_len = std::cmp::min(info.len(), self.win_size.0);
        let line_info = format!(
            "{}/{}",
            self.cursor_controller.cursor_y + 1,
            self.editor_rows.number_of_rows()
        );
        self.editor_contents.push_str(&info[..info_len]);
        for i in info_len..self.win_size.0 {
            if self.win_size.0 - i == line_info.len() {
                self.editor_contents.push_str(&line_info);
                break;
            } else {
                self.editor_contents.push(' ')
            }
        }
        self.editor_contents
            .push_str(&crossterm::style::Attribute::Reset.to_string());
        self.editor_contents.push_str("\r\n");
    }

    fn draw_message_bar(&mut self) {
        crossterm::queue!(
            self.editor_contents,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine)
        )
        .unwrap();
        if let Some(msg) = self.status_message.message() {
            self.editor_contents
                .push_str(&msg[..std::cmp::min(self.win_size.0, msg.len())]);
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
            self.editor_contents.push_str("\r\n");
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
        self.draw_status_bar();
        self.draw_message_bar();
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

    fn insert_char(&mut self, ch: char) {
        if self
            .cursor_controller
            .cursor_y
            .eq(&self.editor_rows.number_of_rows())
        {
            self.editor_rows.insert_row()
        }
        self.editor_rows
            .get_editor_row_mut(self.cursor_controller.cursor_y)
            .insert_char(self.cursor_controller.cursor_x, ch);
        self.cursor_controller.cursor_x += 1;
    }
}

struct Editor {
    reader: Reader,
    output: Output,
}

impl Editor {
    fn new(content: &str, filename: Option<std::path::PathBuf>) -> Self {
        Self {
            reader: Reader,
            output: Output::new(content, filename),
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
            } => {
                if matches!(val, crossterm::event::KeyCode::PageUp) {
                    self.output.cursor_controller.cursor_y =
                        self.output.cursor_controller.row_offset
                } else {
                    self.output.cursor_controller.cursor_y = std::cmp::min(
                        self.output.win_size.1 + self.output.cursor_controller.row_offset - 1,
                        self.output.editor_rows.number_of_rows(),
                    );
                }
                (0..self.output.win_size.1).for_each(|_| {
                    self.output
                        .move_cursor(if matches!(val, crossterm::event::KeyCode::PageUp) {
                            crossterm::event::KeyCode::Up
                        } else {
                            crossterm::event::KeyCode::Down
                        });
                })
            }
            crossterm::event::KeyEvent {
                code: code @ (crossterm::event::KeyCode::Char(..) | crossterm::event::KeyCode::Tab),
                modifiers:
                    crossterm::event::KeyModifiers::NONE | crossterm::event::KeyModifiers::SHIFT,
            } => self.output.insert_char(match code {
                crossterm::event::KeyCode::Tab => '\t',
                crossterm::event::KeyCode::Char(ch) => ch,
                _ => unreachable!(),
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

pub(crate) fn editor(content: &str, filename: Option<std::path::PathBuf>) -> fpm::Result<()> {
    editor_(content, filename).map_err(|e| fpm::Error::UsageError {
        message: e.to_string(),
    })
}

fn editor_(content: &str, filename: Option<std::path::PathBuf>) -> crossterm::Result<()> {
    let _clean_up = CleanUp;
    crossterm::terminal::enable_raw_mode()?;
    let mut editor = Editor::new(content, filename);
    while editor.run()? {}
    Ok(())
}
