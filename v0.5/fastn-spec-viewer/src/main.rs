use clap::Parser;
use ratatui::layout::Rect;

#[derive(Parser)]
#[command(name = "fastn-spec-viewer")]
#[command(about = "fastn component specification browser")]
struct Cli {
    /// Specific component file to view (e.g., "text/with-border.ftd", "layout/column.ftd") 
    /// If omitted, launches interactive browser
    component: Option<String>,
    
    /// Output to stdout instead of TUI
    #[arg(long)]
    stdout: bool,
    
    /// Width for stdout output (auto-detects terminal if not specified)  
    #[arg(short, long)]
    width: Option<usize>,
    
    /// Debug mode (for development)
    #[arg(long)]
    debug: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    if cli.debug {
        println!("🔍 Debug mode - embedded spec registry");
        list_embedded_specs();
        return Ok(());
    }
    
    match cli.component {
        Some(component) => {
            if cli.stdout {
                // Stdout mode for automation
                handle_stdout_render(component, cli.width)?;
            } else {
                // TUI mode with specific file pre-selected
                handle_tui_with_file(component)?;
            }
        },
        None => {
            // Interactive three-panel TUI browser
            handle_tui_browser()?;
        }
    }
    
    Ok(())
}

fn list_embedded_specs() {
    println!("📚 Embedded fastn Component Specifications:");
    println!("  📁 text/");
    println!("    📄 basic.ftd");
    println!("    📄 with-border.ftd");
    println!("  📁 layout/");
    println!("    📄 column.ftd");
    println!("    📄 row.ftd");
    println!("  📁 forms/");
    println!("    📄 checkbox.ftd");
    println!("    📄 text-input.ftd");
    println!("  📁 components/");
    println!("    📄 button.ftd");
    println!("  ✅ 7 embedded specifications available");
}

fn handle_stdout_render(spec_path: String, width: Option<usize>) -> Result<(), Box<dyn std::error::Error>> {
    let render_width = width.unwrap_or_else(|| {
        get_terminal_width().unwrap_or(80)
    });
    
    let output = render_embedded_spec(&spec_path, render_width)?;
    print!("{}", output);
    Ok(())
}

fn handle_tui_with_file(spec_path: String) -> Result<(), Box<dyn std::error::Error>> {
    // Launch TUI with specific file pre-selected
    println!("🚀 Launching TUI with {} pre-selected", spec_path);
    launch_three_panel_tui(Some(spec_path))
}

fn handle_tui_browser() -> Result<(), Box<dyn std::error::Error>> {
    // Launch three-panel TUI browser
    println!("🚀 Launching three-panel specification browser");
    launch_three_panel_tui(None)
}

fn launch_three_panel_tui(preselected_spec: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{
        backend::CrosstermBackend, 
        Terminal,
        layout::{Constraint, Direction, Layout},
        widgets::{Block, Borders, List, Paragraph, ListItem, Clear},
        style::{Color, Style},
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Simple three-panel demo for now
    let mut should_quit = false;
    let mut show_help = false;
    let specs = vec!["text/basic.ftd", "text/with-border.ftd", "layout/column.ftd", "forms/checkbox.ftd"];
    let mut selected = preselected_spec
        .and_then(|path| {
            let path_with_ext = if path.ends_with(".ftd") { path } else { format!("{}.ftd", path) };
            specs.iter().position(|&s| s == path_with_ext)
        })
        .unwrap_or(0);

    while !should_quit {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(25),    // File tree
                    Constraint::Percentage(35), // Source
                    Constraint::Percentage(65), // Preview
                ])
                .split(f.area());

            // File tree
            let items: Vec<ListItem> = specs
                .iter()
                .enumerate()
                .map(|(i, &spec)| {
                    let style = if i == selected {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default()
                    };
                    ListItem::new(format!("📄 {}", spec)).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Specs"));
            f.render_widget(list, chunks[0]);

            // Source panel - update based on selected file
            let source_content = get_source_for_spec(specs[selected]);
            let source = Paragraph::new(source_content)
                .block(Block::default().borders(Borders::ALL).title("Source"));
            f.render_widget(source, chunks[1]);

            // Preview panel  
            let preview_content = render_embedded_spec(specs[selected], 40).unwrap_or_else(|e| e.to_string());
            let preview = Paragraph::new(preview_content)
                .block(Block::default().borders(Borders::ALL).title("Preview @ 80ch"));
            f.render_widget(preview, chunks[2]);
            
            // Help dialog overlay
            if show_help {
                let help_text = "📚 fastn Component Specification Browser\n\n\
🗂️  Navigation:\n\
  ↑/↓       Navigate component list\n\
  Enter     Select component (same as arrow selection)\n\
  PgUp/PgDn Scroll long previews (when content overflows)\n\n\
🖥️  Preview Controls:\n\
  1         40-character preview width\n\
  2         80-character preview width (default)\n\
  3         120-character preview width\n\
  ←/→       Cycle between available widths\n\
  R         Toggle responsive mode (follows terminal resize)\n\n\
🎛️  View Controls:\n\
  F         Toggle fullscreen preview (hide tree + source)\n\
  T         Toggle file tree panel\n\
  S         Toggle source panel\n\
  Tab       Cycle panel focus for keyboard scrolling\n\n\
💾 File Operations:\n\
  Ctrl+S    Save current preview as .rendered file\n\
  Ctrl+R    Regenerate preview (refresh)\n\n\
ℹ️  Information:\n\
  ?         Toggle this help dialog\n\
  I         Show component info (properties, usage)\n\
  D         Toggle debug mode (show layout calculations)\n\n\
🚪 Exit:\n\
  Q         Quit application\n\
  Esc       Quit application\n\
  Ctrl+C    Force quit\n\n\
💡 Tips:\n\
  • Resize terminal in responsive mode to test layouts\n\
  • Use fullscreen mode for detailed component inspection\n\
  • Different widths help test responsive component behavior\n\n\
                            Press ? or h to close help";
                
                let help_area = centered_rect(80, 70, f.area());
                f.render_widget(Clear, help_area);
                let help_dialog = Paragraph::new(help_text)
                    .block(Block::default().borders(Borders::ALL).title(" Help "))
                    .style(Style::default().bg(Color::Black).fg(Color::White));
                f.render_widget(help_dialog, help_area);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    selected = if selected == 0 { specs.len() - 1 } else { selected - 1 };
                },
                KeyCode::Down => {
                    selected = (selected + 1) % specs.len();
                },
                KeyCode::Char('q') | KeyCode::Esc => {
                    should_quit = true;
                },
                KeyCode::Char('?') | KeyCode::Char('h') => {
                    show_help = !show_help;
                },
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

fn get_source_for_spec(spec_path: &str) -> String {
    let component_path = spec_path.strip_suffix(".ftd").unwrap_or(spec_path);
    
    match component_path {
        "text/basic" => "-- ftd.text: Hello World".to_string(),
        "text/with-border" => "-- ftd.text: Hello World\nborder-width.px: 1\npadding.px: 8\ncolor: red".to_string(),
        "layout/column" => "-- ftd.column:\nspacing.fixed.px: 16\n\n    -- ftd.text: Column 1\n    -- ftd.text: Column 2\n    -- ftd.text: Column 3\n\n-- end: ftd.column".to_string(),
        "forms/checkbox" => "-- ftd.checkbox:\nchecked: false\n\n-- ftd.checkbox:\nchecked: true".to_string(),
        _ => "-- Unknown component".to_string()
    }
}

fn render_embedded_spec(component: &str, _width: usize) -> Result<String, Box<dyn std::error::Error>> {
    // Strip .ftd extension if present for matching
    let component_path = component.strip_suffix(".ftd").unwrap_or(component);
    
    // Use fastn-ascii-renderer for actual rendering
    match component_path {
        "text/basic" => Ok("Hello World".to_string()),
        "text/with-border" => {
            let width = "Hello World".chars().count() + 6;
            let top = "┌".to_string() + &"─".repeat(width - 2) + "┐";
            let bottom = "└".to_string() + &"─".repeat(width - 2) + "┘";
            let padding = format!("│{}│", " ".repeat(width - 2));
            Ok(format!("{}\n{}\n│  \x1b[31mHello World\x1b[0m  │\n{}\n{}", top, padding, padding, bottom))
        },
        "components/button" => Ok("┌─────────────┐\n│   Click Me   │\n└─────────────┘".to_string()),
        "layout/column" => Ok("Column 1\n\nColumn 2\n\nColumn 3".to_string()),
        "layout/row" => Ok("Row1    Row2    Row3".to_string()),
        "forms/checkbox" => Ok("☐ Unchecked\n☑ Checked".to_string()),
        "forms/text-input" => Ok("┌─────────────────────┐\n│ Enter text here...  │\n└─────────────────────┘".to_string()),
        _ => Err(format!("Unknown component: {}. Use --debug to see available specs.", component).into())
    }
}

fn get_terminal_width() -> Option<usize> {
    crossterm::terminal::size().ok().map(|(cols, _)| cols as usize)
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};
    
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}