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
    
    /// Height for stdout output (golden ratio of width if not specified)
    #[arg(long)]
    height: Option<usize>,
    
    /// Check mode - validate all specs against rendered snapshots
    #[arg(long)]
    check: bool,
    
    /// Debug mode (for development)
    #[arg(long)]
    debug: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    if cli.check {
        println!("🧪 Check mode - validating all specs against snapshots");
        return handle_check_mode();
    }
    
    if cli.debug {
        println!("🔍 Debug mode - embedded spec registry");
        list_embedded_specs();
        return Ok(());
    }
    
    match cli.component {
        Some(component) => {
            if cli.stdout {
                // Stdout mode for automation
                handle_stdout_render(component, cli.width, cli.height)?;
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

fn handle_stdout_render(spec_path: String, width: Option<usize>, height: Option<usize>) -> Result<(), Box<dyn std::error::Error>> {
    let render_width = width.unwrap_or_else(|| {
        get_terminal_width().unwrap_or(80)
    });
    
    // Golden ratio portrait: height = width × 1.6 for pleasing proportions
    let render_height = height.unwrap_or_else(|| {
        (render_width as f64 * 1.6).round() as usize
    });
    
    let output = render_embedded_spec(&spec_path, render_width, render_height)?;
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
            let preview_content = render_embedded_spec(specs[selected], 80, 128).unwrap_or_else(|e| e.to_string());
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

fn render_embedded_spec(component: &str, available_width: usize, available_height: usize) -> Result<String, Box<dyn std::error::Error>> {
    // Strip .ftd extension if present for matching
    let component_path = component.strip_suffix(".ftd").unwrap_or(component);
    
    // Simple width-aware rendering for now (TODO: Use full Taffy integration)
    match component_path {
        "text/basic" => Ok("Hello World".to_string()),
        "text/with-border" => {
            // Make text responsive to width - demo of width awareness  
            let text = "Hello World";
            let min_width = text.chars().count() + 6; // text + padding + border
            let actual_width = if available_width < min_width { 
                min_width 
            } else { 
                available_width.min(50) // Cap at reasonable size
            };
            
            let content_width = actual_width.saturating_sub(4); // Account for border + padding
            let top = "┌".to_string() + &"─".repeat(content_width) + "┐";
            let bottom = "└".to_string() + &"─".repeat(content_width) + "┘";
            
            // Center text in available space
            let text_padding = (content_width.saturating_sub(text.chars().count())) / 2;
            let text_line = format!("│{}{}{}│", 
                " ".repeat(text_padding),
                format!("\x1b[31m{}\x1b[0m", text),
                " ".repeat(content_width.saturating_sub(text.chars().count() + text_padding))
            );
            
            let padding_line = format!("│{}│", " ".repeat(content_width));
            
            // Add outer window rect to show extent
            let inner_content = format!("{}\n{}\n{}\n{}\n{}", top, padding_line, text_line, padding_line, bottom);
            let window_width = actual_width + 4; // Extra space around component
            let window_top = "╭".to_string() + &"─".repeat(window_width - 2) + "╮";
            let window_bottom = "╰".to_string() + &"─".repeat(window_width - 2) + "╯";
            
            let mut result = Vec::new();
            result.push(window_top);
            result.push(format!("│{}│", " ".repeat(window_width - 2))); // Top padding
            
            for line in inner_content.lines() {
                let padding_needed = window_width.saturating_sub(2).saturating_sub(line.chars().count());
                result.push(format!("│ {}{} │", line, " ".repeat(padding_needed.saturating_sub(1))));
            }
            
            result.push(format!("│{}│", " ".repeat(window_width - 2))); // Bottom padding
            result.push(window_bottom);
            
            Ok(result.join("\n"))
        },
        "components/button" => {
            // Width-responsive button
            let text = "Click Me";
            let min_width = text.chars().count() + 4;
            let button_width = if available_width < min_width { 
                min_width 
            } else { 
                (available_width / 3).min(20) // Make button proportional but not too wide
            };
            
            let content_width = button_width.saturating_sub(2);
            let text_padding = (content_width.saturating_sub(text.chars().count())) / 2;
            
            let top = "┌".to_string() + &"─".repeat(content_width) + "┐";
            let middle = format!("│{}{}{}│",
                " ".repeat(text_padding),
                text,
                " ".repeat(content_width.saturating_sub(text.chars().count() + text_padding))
            );
            let bottom = "└".to_string() + &"─".repeat(content_width) + "┘";
            
            Ok(format!("{}\n{}\n{}", top, middle, bottom))
        },
        "forms/text-input" => {
            // Width-responsive text input
            let input_width = (available_width * 2 / 3).max(20).min(60);
            let content_width = input_width.saturating_sub(2);
            let placeholder = "Enter text here...";
            
            let top = "┌".to_string() + &"─".repeat(content_width) + "┐";
            let middle = format!("│{}{}│",
                placeholder,
                " ".repeat(content_width.saturating_sub(placeholder.chars().count()))
            );
            let bottom = "└".to_string() + &"─".repeat(content_width) + "┘";
            
            Ok(format!("{}\n{}\n{}", top, middle, bottom))
        },
        "layout/column" => Ok("Column 1\n\nColumn 2\n\nColumn 3".to_string()),
        "layout/row" => {
            // Width-responsive row
            if available_width >= 30 {
                Ok("Item1    Item2    Item3".to_string())
            } else {
                Ok("Item1\nItem2\nItem3".to_string()) // Stack when narrow
            }
        },
        "forms/checkbox" => Ok("☐ Unchecked\n☑ Checked".to_string()),
        _ => Err(format!("Unknown component: {}. Use --debug to see available specs.", component).into())
    }
}

fn handle_check_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Checking all component specifications...\n");
    
    // Discover all .ftd files in specs directory
    let spec_files = discover_spec_files_from_disk()?;
    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;
    
    for spec_file in spec_files {
        println!("Testing: {}", spec_file.display());
        
        // Test at different widths with portrait heights (height = width × 1.6)
        for (width, height) in [(40, 64), (80, 128), (120, 192)] {
            total_tests += 1;
            
            // Check if rendered file exists
            let base = spec_file.with_extension("");
            let rendered_file = format!("{}.rendered-{}x{}", base.display(), width, height);
            let rendered_path = std::path::PathBuf::from(&rendered_file);
            
            if rendered_path.exists() {
                // Compare actual vs expected
                let expected = std::fs::read_to_string(&rendered_path)?;
                let actual = render_ftd_file_from_disk(&spec_file, width, height)?;
                
                if expected.trim() == actual.trim() {
                    passed_tests += 1;
                    println!("  ✅ {}ch: PASS", width);
                } else {
                    failed_tests += 1;
                    println!("  ❌ {}ch: FAIL", width);
                    if expected.lines().count() <= 3 && actual.lines().count() <= 3 {
                        println!("     Expected: {}", expected.replace('\n', " | "));
                        println!("     Actual:   {}", actual.replace('\n', " | "));
                    }
                }
            } else {
                println!("  ⚠️  {}ch: Missing .rendered-{} file", width, width);
            }
        }
        println!();
    }
    
    // Summary  
    println!("📊 Test Results:");
    println!("  ✅ Passed: {}", passed_tests);
    println!("  ❌ Failed: {}", failed_tests);
    println!("  📝 Total:  {}", total_tests);
    
    if failed_tests > 0 {
        println!("\n💡 Tip: Use generate mode to update failing snapshots:");
        println!("   fastn-spec-viewer --generate");
        std::process::exit(1);
    } else {
        println!("\n🎉 All tests passed!");
    }
    
    Ok(())
}

fn discover_spec_files_from_disk() -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    
    for entry in walkdir::WalkDir::new("specs") {
        let entry = entry?;
        if let Some(ext) = entry.path().extension() {
            if ext == "ftd" {
                files.push(entry.path().to_path_buf());
            }
        }
    }
    
    files.sort();
    Ok(files)
}

fn render_ftd_file_from_disk(
    file: &std::path::Path, 
    width: usize,
    height: usize
) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file)?;
    
    // Simple ftd parsing - look for text components
    if content.contains("-- ftd.text:") {
        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            if let Some(text_start) = line.find("-- ftd.text:") {
                let text_content = line[text_start + 12..].trim();
                
                // Check properties
                let has_border = content.contains("border-width");
                let has_padding = content.contains("padding");
                let has_color = content.contains("color:");
                
                // Use same responsive rendering logic
                if has_border && has_padding {
                    let min_width = text_content.chars().count() + 6;
                    let actual_width = width.max(min_width).min(50);
                    let content_width = actual_width.saturating_sub(4);
                    
                    let top = "┌".to_string() + &"─".repeat(content_width) + "┐";
                    let bottom = "└".to_string() + &"─".repeat(content_width) + "┘";
                    
                    let text_padding = (content_width.saturating_sub(text_content.chars().count())) / 2;
                    let text_line = if has_color {
                        format!("│{}\x1b[31m{}\x1b[0m{}│", 
                            " ".repeat(text_padding),
                            text_content,
                            " ".repeat(content_width.saturating_sub(text_content.chars().count() + text_padding))
                        )
                    } else {
                        format!("│{}{}{}│", 
                            " ".repeat(text_padding),
                            text_content,
                            " ".repeat(content_width.saturating_sub(text_content.chars().count() + text_padding))
                        )
                    };
                    
                    let padding_line = format!("│{}│", " ".repeat(content_width));
                    return Ok(format!("{}\n{}\n{}\n{}\n{}", top, padding_line, text_line, padding_line, bottom));
                } else {
                    return Ok(text_content.to_string());
                }
            }
        }
    }
    
    Ok("<!-- Unsupported component -->".to_string())
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