use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct SpecViewerApp {
    // File management
    pub specs_dir: PathBuf,
    pub current_file: Option<PathBuf>,
    pub spec_files: Vec<PathBuf>,
    pub selected_index: usize,
    
    // Preview state
    pub current_width: usize,
    pub available_widths: Vec<usize>,
    pub expected_renders: HashMap<usize, String>,
    pub current_render: Option<String>,
    
    // UI state
    pub show_tree: bool,
    pub show_source: bool,
    pub fullscreen: bool,
    pub responsive_mode: bool,
    
    // Status
    pub render_status: RenderStatus,
    pub should_quit: bool,
}

#[derive(Debug, Clone)]
pub enum RenderStatus {
    Loading,
    Match { width_count: usize },
    Mismatch { failed_widths: Vec<usize> },
    ParseError(String),
    NoRenderedFiles,
}

impl SpecViewerApp {
    pub fn new(specs_dir: PathBuf) -> Self {
        Self {
            specs_dir,
            current_file: None,
            spec_files: vec![],
            selected_index: 0,
            
            current_width: 80,
            available_widths: vec![40, 80, 120],
            expected_renders: HashMap::new(),
            current_render: None,
            
            show_tree: true,
            show_source: true,
            fullscreen: false,
            responsive_mode: false,
            
            render_status: RenderStatus::Loading,
            should_quit: false,
        }
    }

    pub fn discover_spec_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.spec_files.clear();
        
        for entry in walkdir::WalkDir::new(&self.specs_dir) {
            let entry = entry?;
            if let Some(ext) = entry.path().extension() {
                if ext == "fastn" {
                    self.spec_files.push(entry.path().to_path_buf());
                }
            }
        }
        
        self.spec_files.sort();
        
        if !self.spec_files.is_empty() && self.current_file.is_none() {
            self.select_file(0)?;
        }
        
        Ok(())
    }

    pub fn select_file(&mut self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        if index < self.spec_files.len() {
            self.selected_index = index;
            self.current_file = Some(self.spec_files[index].clone());
            self.load_expected_renders()?;
            self.regenerate_current_render()?;
        }
        Ok(())
    }

    fn load_expected_renders(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.expected_renders.clear();
        
        if let Some(ref file) = self.current_file {
            let base = file.with_extension("");
            
            for width in &[40, 80, 120] {
                let rendered_path = PathBuf::from(format!("{}.rendered-{}", base.display(), width));
                if rendered_path.exists() {
                    let content = std::fs::read_to_string(&rendered_path)?;
                    self.expected_renders.insert(*width, content);
                }
            }
            
            self.available_widths = self.expected_renders.keys().copied().collect();
            self.available_widths.sort();
            
            // Set current width to first available or default to 80
            if !self.available_widths.is_empty() {
                self.current_width = self.available_widths[0];
            }
        }
        
        Ok(())
    }

    fn regenerate_current_render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref file) = self.current_file {
            // Try to parse and render the fastn file
            match self.parse_and_render_file(file) {
                Ok(rendered_output) => {
                    self.current_render = Some(rendered_output);
                    self.render_status = RenderStatus::Loading; // Will be updated by update_render_status
                },
                Err(e) => {
                    self.render_status = RenderStatus::ParseError(e.to_string());
                    self.current_render = Some(format!("Parse Error:\n{}", e));
                }
            }
            
            self.update_render_status();
        }
        Ok(())
    }

    fn parse_and_render_file(&self, file: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(file)?;
        
        // For Week 3: Simple parsing - just look for text components
        // TODO: Integrate with actual fastn parser when available
        if content.contains("-- ftd.text:") {
            // Extract text content (very simplified)
            let lines: Vec<&str> = content.lines().collect();
            for line in lines {
                if let Some(text_start) = line.find("-- ftd.text:") {
                    let text_content = line[text_start + 12..].trim();
                    
                    // Check for border and padding
                    let has_border = content.contains("border-width");
                    let has_padding = content.contains("padding");
                    let has_color = content.contains("color:");
                    
                    return Ok(self.render_simple_text(text_content, has_border, has_padding, has_color));
                }
            }
        }
        
        Ok("<!-- Unsupported component type -->".to_string())
    }

    fn render_simple_text(&self, text: &str, has_border: bool, has_padding: bool, has_color: bool) -> String {
        if has_border && has_padding {
            // Render with border and padding
            let width = text.chars().count() + 6; // text + padding + border
            let top_border = "┌".to_string() + &"─".repeat(width - 2) + "┐";
            let bottom_border = "└".to_string() + &"─".repeat(width - 2) + "┘";
            let content_line = format!("│  {}  │", text);
            let padding_line = format!("│{}│", " ".repeat(width - 2));
            
            if has_color {
                // Simple ANSI red color for demonstration
                format!("{}\n{}\n│  \x1b[31m{}\x1b[0m  │\n{}\n{}",
                    top_border, padding_line, text, padding_line, bottom_border)
            } else {
                format!("{}\n{}\n{}\n{}\n{}", 
                    top_border, padding_line, content_line, padding_line, bottom_border)
            }
        } else if has_border {
            // Render with border only
            let width = text.chars().count() + 2; 
            let top_border = "┌".to_string() + &"─".repeat(width - 2) + "┐";
            let bottom_border = "└".to_string() + &"─".repeat(width - 2) + "┘";
            let content_line = format!("│{}│", text);
            
            if has_color {
                format!("{}\n│\x1b[31m{}\x1b[0m│\n{}", top_border, text, bottom_border)
            } else {
                format!("{}\n{}\n{}", top_border, content_line, bottom_border)
            }
        } else {
            // Plain text
            if has_color {
                format!("\x1b[31m{}\x1b[0m", text)
            } else {
                text.to_string()
            }
        }
    }

    fn update_render_status(&mut self) {
        if self.expected_renders.is_empty() {
            self.render_status = RenderStatus::NoRenderedFiles;
        } else if let Some(ref current) = self.current_render {
            let mut failed_widths = vec![];
            
            for (width, expected) in &self.expected_renders {
                // TODO: Generate render at specific width and compare
                // For now, assume match
                if current.trim() != expected.trim() {
                    failed_widths.push(*width);
                }
            }
            
            if failed_widths.is_empty() {
                self.render_status = RenderStatus::Match { 
                    width_count: self.expected_renders.len() 
                };
            } else {
                self.render_status = RenderStatus::Mismatch { failed_widths };
            }
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        match key.code {
            // File navigation
            KeyCode::Up => self.previous_file(),
            KeyCode::Down => self.next_file(),
            KeyCode::Enter => { /* File already selected by up/down */ },
            
            // Width switching
            KeyCode::Left => self.previous_width(),
            KeyCode::Right => self.next_width(),
            KeyCode::Char('1') => self.set_width(40),
            KeyCode::Char('2') => self.set_width(80),
            KeyCode::Char('3') => self.set_width(120),
            
            // Mode switching
            KeyCode::Char('f') | KeyCode::Char('F') => self.toggle_fullscreen(),
            KeyCode::Char('r') | KeyCode::Char('R') => self.toggle_responsive_mode(),
            KeyCode::Char('t') | KeyCode::Char('T') => self.toggle_tree_panel(),
            
            // File operations  
            KeyCode::Char('s') => self.save_current_render()?,
            
            // Quit
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                self.should_quit = true;
            },
            
            _ => {}
        }
        Ok(())
    }

    fn previous_file(&mut self) {
        if !self.spec_files.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.spec_files.len() - 1
            } else {
                self.selected_index - 1
            };
            let _ = self.select_file(self.selected_index);
        }
    }

    fn next_file(&mut self) {
        if !self.spec_files.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.spec_files.len();
            let _ = self.select_file(self.selected_index);
        }
    }

    fn previous_width(&mut self) {
        if let Some(current_pos) = self.available_widths.iter().position(|&w| w == self.current_width) {
            if current_pos > 0 {
                self.current_width = self.available_widths[current_pos - 1];
                let _ = self.regenerate_current_render();
            }
        }
    }

    fn next_width(&mut self) {
        if let Some(current_pos) = self.available_widths.iter().position(|&w| w == self.current_width) {
            if current_pos + 1 < self.available_widths.len() {
                self.current_width = self.available_widths[current_pos + 1];
                let _ = self.regenerate_current_render();
            }
        }
    }

    fn set_width(&mut self, width: usize) {
        if self.available_widths.contains(&width) {
            self.current_width = width;
            let _ = self.regenerate_current_render();
        }
    }

    fn toggle_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
        if self.fullscreen {
            self.show_tree = false;
            self.show_source = false;
        } else {
            self.show_tree = true;
            self.show_source = true;
        }
    }

    fn toggle_responsive_mode(&mut self) {
        self.responsive_mode = !self.responsive_mode;
        let _ = self.regenerate_current_render();
    }

    fn toggle_tree_panel(&mut self) {
        self.show_tree = !self.show_tree;
    }

    fn save_current_render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(file), Some(current)) = (&self.current_file, &self.current_render) {
            let base = file.with_extension("");
            let rendered_path = PathBuf::from(format!("{}.rendered-{}", base.display(), self.current_width));
            
            std::fs::write(&rendered_path, current)?;
            
            // Reload expected renders to include newly saved file
            self.load_expected_renders()?;
        }
        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame) {
        let size = frame.area();
        
        if self.fullscreen {
            self.draw_fullscreen_preview(frame, size);
        } else {
            self.draw_three_panel_layout(frame, size);
        }
    }

    fn draw_three_panel_layout(&self, frame: &mut Frame, area: Rect) {
        let constraints = if self.show_tree && self.show_source {
            vec![
                Constraint::Length(25),    // File tree
                Constraint::Percentage(40), // Source
                Constraint::Percentage(60), // Preview  
            ]
        } else if self.show_tree {
            vec![
                Constraint::Length(25),    // File tree
                Constraint::Percentage(100), // Preview only
            ]
        } else if self.show_source {
            vec![
                Constraint::Percentage(40), // Source
                Constraint::Percentage(60), // Preview
            ]
        } else {
            vec![Constraint::Percentage(100)] // Preview only
        };

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        let mut chunk_index = 0;

        // File tree panel
        if self.show_tree {
            self.draw_file_tree(frame, chunks[chunk_index]);
            chunk_index += 1;
        }

        // Source panel
        if self.show_source {
            self.draw_source_panel(frame, chunks[chunk_index]);
            chunk_index += 1;
        }

        // Preview panel (always shown)
        self.draw_preview_panel(frame, chunks[chunk_index]);
    }

    fn draw_fullscreen_preview(&self, frame: &mut Frame, area: Rect) {
        self.draw_preview_panel(frame, area);
    }

    fn draw_file_tree(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ratatui::widgets::ListItem> = self.spec_files
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let style = if i == self.selected_index {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                
                let filename = path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();
                    
                ratatui::widgets::ListItem::new(format!("📄 {}", filename)).style(style)
            })
            .collect();

        let list = ratatui::widgets::List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Spec Files"));

        frame.render_widget(list, area);
    }

    fn draw_source_panel(&self, frame: &mut Frame, area: Rect) {
        let content = if let Some(ref file) = self.current_file {
            std::fs::read_to_string(file).unwrap_or_else(|_| "Error reading file".to_string())
        } else {
            "No file selected".to_string()
        };

        let paragraph = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title("Source"));

        frame.render_widget(paragraph, area);
    }

    fn draw_preview_panel(&self, frame: &mut Frame, area: Rect) {
        let title = format!("Preview @ {}ch", self.current_width);
        
        let content = if let Some(ref render) = self.current_render {
            render.clone()
        } else {
            "No render available".to_string()
        };

        let status_text = match &self.render_status {
            RenderStatus::Loading => "🔄 Loading...".to_string(),
            RenderStatus::Match { width_count } => format!("✅ MATCH ({} widths)", width_count),
            RenderStatus::Mismatch { failed_widths } => format!("❌ MISMATCH ({})", failed_widths.len()),
            RenderStatus::ParseError(err) => format!("⚠️ ERROR: {}", err),
            RenderStatus::NoRenderedFiles => "📝 No .rendered files".to_string(),
        };

        let title_with_status = format!("{} - {}", title, status_text);

        let paragraph = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title(title_with_status));

        frame.render_widget(paragraph, area);
        
        // Controls help at bottom
        if !self.fullscreen {
            let controls = "◀▶: Width | F: Fullscreen | R: Responsive | S: Save | Q: Quit";
            let help_area = Rect {
                x: area.x,
                y: area.y + area.height.saturating_sub(1),
                width: area.width,
                height: 1,
            };
            
            let help = Paragraph::new(controls)
                .style(Style::default().fg(Color::Gray));
            
            frame.render_widget(help, help_area);
        }
    }
}

impl Default for SpecViewerApp {
    fn default() -> Self {
        Self::new(PathBuf::from("specs"))
    }
}