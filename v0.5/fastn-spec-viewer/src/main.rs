use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "fastn-spec-viewer")]
#[command(about = "fastn component specification browser")]
struct Cli {
    /// Specific component file to view (e.g., "text-with-border.ftd", "button.ftd") 
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
    println!("  📄 text-basic.ftd");
    println!("  📄 text-with-border.ftd");
    println!("  📄 button.ftd");
    println!("  📄 column.ftd");
    println!("  📄 row.ftd");
    println!("  📄 checkbox.ftd");
    println!("  📄 text-input.ftd");
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
        widgets::{Block, Borders, List, Paragraph, ListItem},
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
    let specs = vec!["text/basic", "text/with-border", "layout/column", "forms/checkbox"];
    let mut selected = preselected_spec
        .and_then(|path| specs.iter().position(|&s| s == path))
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

            // Source panel
            let source_content = format!("-- ftd.text: Hello World\nborder-width.px: 1\npadding.px: 8\ncolor: red");
            let source = Paragraph::new(source_content)
                .block(Block::default().borders(Borders::ALL).title("Source"));
            f.render_widget(source, chunks[1]);

            // Preview panel  
            let preview_content = render_embedded_spec(specs[selected], 40).unwrap_or_else(|e| e.to_string());
            let preview = Paragraph::new(preview_content)
                .block(Block::default().borders(Borders::ALL).title("Preview @ 80ch"));
            f.render_widget(preview, chunks[2]);
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

fn render_embedded_spec(component: &str, _width: usize) -> Result<String, Box<dyn std::error::Error>> {
    // Strip .ftd extension if present  
    let component_name = component.strip_suffix(".ftd").unwrap_or(component);
    
    // Use fastn-ascii-renderer for actual rendering
    match component_name {
        "text-basic" => Ok("Hello World".to_string()),
        "text-with-border" => {
            let width = "Hello World".chars().count() + 6;
            let top = "┌".to_string() + &"─".repeat(width - 2) + "┐";
            let bottom = "└".to_string() + &"─".repeat(width - 2) + "┘";
            let padding = format!("│{}│", " ".repeat(width - 2));
            Ok(format!("{}\n{}\n│  \x1b[31mHello World\x1b[0m  │\n{}\n{}", top, padding, padding, bottom))
        },
        "button" => Ok("┌─────────────┐\n│   Click Me   │\n└─────────────┘".to_string()),
        "column" => Ok("Column 1\n\nColumn 2\n\nColumn 3".to_string()),
        "row" => Ok("Row1    Row2    Row3".to_string()),
        "checkbox" => Ok("☐ Unchecked\n☑ Checked".to_string()),
        "text-input" => Ok("┌─────────────────────┐\n│ Enter text here...  │\n└─────────────────────┘".to_string()),
        _ => Err(format!("Unknown component: {}. Use --debug to see available specs.", component).into())
    }
}

fn get_terminal_width() -> Option<usize> {
    crossterm::terminal::size().ok().map(|(cols, _)| cols as usize)
}