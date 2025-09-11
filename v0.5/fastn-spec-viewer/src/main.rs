use clap::Parser;
use fastn_spec_viewer::{embedded_specs, spec_renderer};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

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

    /// Auto-fix mode - update snapshots for failing tests
    #[arg(long)]
    autofix: bool,

    /// Auto-fix specific component (use with --autofix)
    #[arg(long)]
    autofix_component: Option<String>,

    /// Debug mode (for development)
    #[arg(long)]
    debug: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.check || cli.autofix {
        return handle_check_mode(cli.autofix, cli.autofix_component);
    }

    if cli.debug {
        println!("🔍 Debug mode - embedded spec registry");
        list_embedded_specs();
        return Ok(());
    }

    match cli.component {
        Some(component) => {
            if cli.stdout {
                // Stdout mode using clean DocumentRenderer API
                handle_stdout_render(component, cli.width, cli.height)?;
            } else {
                // TUI mode with specific file pre-selected
                handle_tui_with_file(component)?;
            }
        }
        None => {
            // Interactive three-panel TUI browser
            handle_tui_browser()?;
        }
    }

    Ok(())
}

fn list_embedded_specs() {
    println!("📚 Embedded fastn Document Specifications:");
    for (category, specs) in embedded_specs::get_spec_categories() {
        println!("  📁 {}/", category);
        for spec in specs {
            println!("    📄 {}", spec);
        }
    }
    println!(
        "  ✅ {} embedded specifications available",
        embedded_specs::list_embedded_specs().len()
    );
}

fn handle_stdout_render(
    spec_path: String,
    width: Option<usize>,
    height: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let render_width = width.unwrap_or_else(|| get_terminal_width().unwrap_or(80));

    // Golden ratio portrait: height = width × 1.6 for pleasing proportions
    let render_height = height.unwrap_or_else(|| (render_width as f64 * 1.6).round() as usize);

    // Use clean DocumentRenderer API
    let spec_output = spec_renderer::render_spec(&spec_path, render_width, render_height)?;
    print!("{}", spec_output.terminal_display());
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

fn launch_three_panel_tui(
    preselected_spec: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use ratatui::{
        Terminal,
        backend::CrosstermBackend,
        layout::{Constraint, Direction, Layout},
        style::{Color, Style},
        widgets::{Block, Borders, List, ListItem, Paragraph},
    };

    // Setup terminal
    if let Err(e) = enable_raw_mode() {
        eprintln!("Failed to enable raw mode: {}", e);
        eprintln!("Try using --stdout flag for non-interactive output.");
        std::process::exit(1);
    }

    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // TUI state
    let specs = embedded_specs::list_embedded_specs();
    let mut selected = preselected_spec
        .and_then(|path| {
            let path_with_ext = if path.ends_with(".ftd") {
                path
            } else {
                format!("{}.ftd", path)
            };
            specs.iter().position(|&s| s == path_with_ext)
        })
        .unwrap_or(0);
    let mut should_quit = false;
    let mut show_help = false;

    while !should_quit {
        terminal.draw(|f| {
            if show_help {
                draw_help_overlay(f);
                return;
            }

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(25),     // File tree
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

            let list =
                List::new(items).block(Block::default().borders(Borders::ALL).title("Specs"));
            f.render_widget(list, chunks[0]);

            // Source panel - show actual embedded spec source
            let source_content = embedded_specs::get_embedded_spec(specs[selected])
                .unwrap_or_else(|e| format!("Error: {}", e));
            let source = Paragraph::new(source_content)
                .block(Block::default().borders(Borders::ALL).title("Source"));
            f.render_widget(source, chunks[1]);

            // Preview panel using clean DocumentRenderer API
            let preview_content = spec_renderer::render_spec(specs[selected], 80, 128)
                .map(|output| output.ansi_version)
                .unwrap_or_else(|e| format!("Render Error: {}", e));
            let preview = Paragraph::new(preview_content).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Preview @ 80ch"),
            );
            f.render_widget(preview, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    selected = if selected == 0 {
                        specs.len() - 1
                    } else {
                        selected - 1
                    };
                }
                KeyCode::Down => {
                    selected = (selected + 1) % specs.len();
                }
                KeyCode::Char('?') | KeyCode::Char('h') => {
                    show_help = !show_help;
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    should_quit = true;
                }
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn draw_help_overlay(f: &mut ratatui::Frame) {
    let help_text = "📚 fastn Document Specification Browser\n\n\
🗂️  Navigation:\n\
  ↑/↓       Navigate document list\n\
  Enter     Select document\n\n\
🖥️  Preview Controls:\n\
  1         40-character preview width\n\
  2         80-character preview width (default)\n\
  3         120-character preview width\n\n\
ℹ️  Information:\n\
  ?         Toggle this help dialog\n\n\
🚪 Exit:\n\
  Q         Quit application\n\
  Esc       Quit application\n\n\
                            Press ? or h to close help";

    let help_area = centered_rect(80, 70, f.area());
    f.render_widget(ratatui::widgets::Clear, help_area);
    let help_dialog = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .style(Style::default().bg(Color::Black).fg(Color::White));
    f.render_widget(help_dialog, help_area);
}

fn handle_check_mode(
    autofix: bool,
    autofix_component: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if autofix {
        println!("🔧 Auto-fix mode - updating snapshots...\n");
    } else {
        println!("🧪 Checking all component specifications...\n");
    }

    // Discover all .ftd files in specs directory
    let spec_files = discover_spec_files_from_disk()?;
    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;
    let mut fixed_tests = 0;

    for spec_file in spec_files {
        println!("Testing: {}", spec_file.display());

        total_tests += 1;

        // Single .rendered file contains all dimensions
        let base = spec_file.with_extension("");
        let rendered_file = format!("{}.rendered", base.display());
        let rendered_path = std::path::PathBuf::from(&rendered_file);

        if rendered_path.exists() {
            // Compare actual vs expected using clean API
            let expected = std::fs::read_to_string(&rendered_path)?;
            let file_path_str = spec_file.to_string_lossy();
            let spec_path = file_path_str
                .trim_start_matches("specs/")
                .trim_end_matches(".ftd");
            let actual = spec_renderer::render_all_dimensions(spec_path)?;

            if expected.trim() == actual.trim() {
                passed_tests += 1;
                println!("  ✅ All dimensions: PASS");
            } else {
                failed_tests += 1;
                println!("  ❌ All dimensions: FAIL");

                // Auto-fix if requested
                if autofix && should_fix_component(&spec_file, &autofix_component) {
                    std::fs::write(&rendered_path, &actual)?;
                    fixed_tests += 1;
                    println!("  🔧 All dimensions: FIXED - updated snapshot");
                }
            }
        } else {
            println!("  ⚠️  Missing .rendered file");

            // Auto-create missing file if in autofix mode
            if autofix && should_fix_component(&spec_file, &autofix_component) {
                let file_path_str = spec_file.to_string_lossy();
                let spec_path = file_path_str
                    .trim_start_matches("specs/")
                    .trim_end_matches(".ftd");

                let all_dimensions = spec_renderer::render_all_dimensions(spec_path)?;
                std::fs::write(&rendered_path, &all_dimensions)?;
                fixed_tests += 1;
                println!("  🔧 CREATED - generated complete rendered file");
            }
        }
        println!();
    }

    // Summary reporting
    if autofix {
        println!("📊 Auto-fix Results:");
        println!("  ✅ Passed: {}", passed_tests);
        println!("  🔧 Fixed: {}", fixed_tests);
        println!(
            "  ❌ Failed: {}",
            (failed_tests as i32).saturating_sub(fixed_tests as i32)
        );
        println!("  📝 Total:  {}", total_tests);

        if fixed_tests > 0 {
            println!("\n🔧 Updated {} snapshot(s)", fixed_tests);
        }
    } else {
        println!("📊 Test Results:");
        println!("  ✅ Passed: {}", passed_tests);
        println!("  ❌ Failed: {}", failed_tests);
        println!("  📝 Total:  {}", total_tests);

        if failed_tests > 0 {
            println!("\n💡 Tip: Use auto-fix to update snapshots:");
            println!("   fastn-spec-viewer --autofix");
            std::process::exit(1);
        } else {
            println!("\n🎉 All tests passed!");
        }
    }

    Ok(())
}

// Helper functions
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

fn should_fix_component(spec_file: &std::path::Path, autofix_component: &Option<String>) -> bool {
    match autofix_component {
        Some(target_component) => {
            if let Some(file_name) = spec_file.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    return name_str.starts_with(target_component)
                        || spec_file.to_string_lossy().contains(target_component);
                }
            }
            false
        }
        None => true,
    }
}

fn get_terminal_width() -> Option<usize> {
    crossterm::terminal::size()
        .ok()
        .map(|(cols, _)| cols as usize)
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
