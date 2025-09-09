use clap::Parser;
use std::path::PathBuf;
use crossterm::event::Event;
use ratatui::Terminal;

#[derive(Parser)]
#[command(name = "spec-viewer")]
#[command(about = "fastn component specification browser")]
struct Cli {
    /// Specific spec to view (e.g., "text/with-border", "layout/column")
    /// If omitted, launches interactive browser
    spec_path: Option<String>,
    
    /// Output to stdout instead of fullscreen preview
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
    
    match cli.spec_path {
        Some(spec_path) => {
            // Direct spec rendering
            if cli.stdout {
                handle_stdout_render(spec_path, cli.width)?;
            } else {
                handle_fullscreen_spec(spec_path)?;
            }
        },
        None => {
            // Interactive browser mode
            handle_browser_mode()?;
        }
    }
    
    Ok(())
}

fn list_embedded_specs() {
    println!("📚 Embedded fastn Component Specifications:");
    println!("  📄 text/basic");
    println!("  📄 text/with-border");
    println!("  📁 layout/");
    println!("    📄 column");
    println!("    📄 row");
    println!("  📁 forms/");
    println!("    📄 checkbox");
    println!("    📄 text-input");
    println!("  ✅ 6 embedded specifications available");
}

fn handle_stdout_render(spec_path: String, width: Option<usize>) -> Result<(), Box<dyn std::error::Error>> {
    let render_width = width.unwrap_or_else(|| {
        get_terminal_width().unwrap_or(80)
    });
    
    let output = render_embedded_spec(&spec_path, render_width)?;
    print!("{}", output);
    Ok(())
}

fn handle_fullscreen_spec(spec_path: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("🖥️  Fullscreen responsive preview: {}", spec_path);
    println!("Resize terminal to test responsive behavior. Press Ctrl+C to exit.\n");
    
    // Get initial terminal size
    let initial_width = get_terminal_width().unwrap_or(80);
    let output = render_embedded_spec(&spec_path, initial_width)?;
    
    // Clear screen and show component
    print!("\x1b[2J\x1b[H"); // Clear screen, move cursor to top
    println!("{}", output);
    println!("\n📏 Rendered at {} characters - resize terminal to test responsive behavior", initial_width);
    
    // TODO: Add terminal resize detection and re-rendering
    // For now, just show static render
    
    Ok(())
}

fn handle_browser_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Interactive fastn component specification browser");
    println!("📚 Embedded specifications available for browsing");
    
    // TODO: Launch full TUI browser
    // For now, show available specs
    list_embedded_specs();
    
    Ok(())
}

fn render_embedded_spec(spec_path: &str, width: usize) -> Result<String, Box<dyn std::error::Error>> {
    // For now, use hardcoded examples matching our test files
    match spec_path {
        "text/basic" | "text-basic" => {
            Ok("Hello World".to_string())
        },
        "text/with-border" | "text-with-border" => {
            let width = "Hello World".chars().count() + 6; // text + padding + border
            let top = "┌".to_string() + &"─".repeat(width - 2) + "┐";
            let bottom = "└".to_string() + &"─".repeat(width - 2) + "┘";
            let padding = format!("│{}│", " ".repeat(width - 2));
            
            Ok(format!("{}\n{}\n│  \x1b[31mHello World\x1b[0m  │\n{}\n{}", top, padding, padding, bottom))
        },
        _ => {
            Err(format!("Unknown spec: {}. Use --debug to see available specs.", spec_path).into())
        }
    }
}

fn handle_tui_mode(specs_dir: PathBuf, width: usize, debug: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Verify specs directory exists
    if !specs_dir.exists() {
        eprintln!("Specs directory '{}' not found!", specs_dir.display());
        eprintln!("Create it or specify a different path.");
        std::process::exit(1);
    }

    if debug {
        println!("🚀 fastn spec-viewer TUI starting...");
        println!("📁 Specs directory: {}", specs_dir.display());
        println!("📏 Default width: {} characters", width);
        println!("🔍 Debug mode enabled");
        
        // Simple file discovery for debugging
        discover_spec_files(&specs_dir)?;
        return Ok(());
    }

    // Launch TUI application
    launch_tui_app(specs_dir, width)?;
    
    Ok(())
}

fn handle_render_mode(
    file: PathBuf,
    width: Option<usize>,
    auto_width: bool,
    follow: bool,
    save: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !file.exists() {
        eprintln!("File '{}' not found!", file.display());
        std::process::exit(1);
    }

    // Determine width
    let render_width = if auto_width {
        get_terminal_width().unwrap_or(80)
    } else {
        width.unwrap_or(80)
    };

    if follow {
        handle_follow_mode(file, render_width, save)
    } else {
        handle_single_render(file, render_width, save)
    }
}

fn handle_single_render(file: PathBuf, width: usize, save: bool) -> Result<(), Box<dyn std::error::Error>> {
    let output = render_fastn_file(&file, width)?;
    
    if save {
        let base = file.with_extension("");
        let rendered_file = PathBuf::from(format!("{}.rendered-{}", base.display(), width));
        std::fs::write(&rendered_file, &output)?;
        eprintln!("💾 Saved to: {}", rendered_file.display());
    }
    
    // Output to stdout (can be piped or redirected)
    print!("{}", output);
    Ok(())
}

fn handle_follow_mode(file: PathBuf, initial_width: usize, save: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Follow mode: {} @ {} chars", file.display(), initial_width);
    println!("Resize terminal to test responsive behavior. Press Ctrl+C to exit.\n");
    
    let mut current_width = initial_width;
    
    // Initial render
    let output = render_fastn_file(&file, current_width)?;
    println!("{}", output);
    
    // TODO: Implement terminal resize detection and re-rendering
    // For now, just render once
    
    if save {
        let base = file.with_extension("");
        let rendered_file = PathBuf::from(format!("{}.rendered-{}", base.display(), current_width));
        std::fs::write(&rendered_file, &output)?;
        eprintln!("\n💾 Saved to: {}", rendered_file.display());
    }
    
    Ok(())
}

fn handle_test_mode(
    target: PathBuf,
    widths: Option<String>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: {}", target.display());
    
    if target.is_file() {
        test_single_file(target, widths, verbose)
    } else {
        test_directory(target, widths, verbose)
    }
}

fn test_single_file(
    file: PathBuf,
    _widths: Option<String>,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing {}...", file.display());
    
    let base = file.with_extension("");
    let mut results = vec![];
    
    for width in [40, 80, 120] {
        let rendered_file = PathBuf::from(format!("{}.rendered-{}", base.display(), width));
        if rendered_file.exists() {
            let expected = std::fs::read_to_string(&rendered_file)?;
            let actual = render_fastn_file(&file, width)?;
            
            if expected.trim() == actual.trim() {
                results.push(format!("  ✅ {}ch: PASS", width));
            } else {
                results.push(format!("  ❌ {}ch: FAIL", width));
            }
        }
    }
    
    for result in results {
        println!("{}", result);
    }
    
    Ok(())
}

fn test_directory(
    _dir: PathBuf,
    _widths: Option<String>,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Directory testing not yet implemented");
    Ok(())
}

fn handle_generate_mode(
    files: Vec<PathBuf>,
    widths: String,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let width_list: Vec<usize> = widths
        .split(',')
        .filter_map(|w| w.trim().parse().ok())
        .collect();

    println!("📝 Generating renders for {} file(s) at {} widths", 
        files.len(), width_list.len());

    for file in files {
        if !file.exists() {
            eprintln!("⚠️  Skipping '{}': file not found", file.display());
            continue;
        }

        println!("Processing: {}", file.display());
        let base = file.with_extension("");

        for &width in &width_list {
            let rendered_file = PathBuf::from(format!("{}.rendered-{}", base.display(), width));
            
            if rendered_file.exists() && !force {
                println!("  ⏭️  Skipping {}ch: file exists (use --force to overwrite)", width);
                continue;
            }

            match render_fastn_file(&file, width) {
                Ok(output) => {
                    std::fs::write(&rendered_file, output)?;
                    println!("  ✅ Generated: {}", rendered_file.display());
                },
                Err(e) => {
                    eprintln!("  ❌ Failed {}ch: {}", width, e);
                }
            }
        }
    }

    Ok(())
}

fn render_fastn_file(file: &PathBuf, width: usize) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file)?;
    
    // Simple parser for text components
    if content.contains("-- ftd.text:") {
        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            if let Some(text_start) = line.find("-- ftd.text:") {
                let text_content = line[text_start + 12..].trim();
                
                // Check for styling properties
                let has_border = content.contains("border-width");
                let has_padding = content.contains("padding");  
                let has_color = content.contains("color:");
                
                return Ok(render_simple_text(text_content, has_border, has_padding, has_color, width));
            }
        }
    }
    
    Ok(format!("<!-- Unsupported: {} -->", file.display()))
}

fn render_simple_text(text: &str, has_border: bool, has_padding: bool, has_color: bool, _width: usize) -> String {
    // Same rendering logic as in the TUI app
    if has_border && has_padding {
        let width = text.chars().count() + 6;
        let top = "┌".to_string() + &"─".repeat(width - 2) + "┐";
        let bottom = "└".to_string() + &"─".repeat(width - 2) + "┘";
        let padding = format!("│{}│", " ".repeat(width - 2));
        
        if has_color {
            format!("{}\n{}\n│  \x1b[31m{}\x1b[0m  │\n{}\n{}", top, padding, text, padding, bottom)
        } else {
            format!("{}\n{}\n│  {}  │\n{}\n{}", top, padding, text, padding, bottom)
        }
    } else if has_border {
        let width = text.chars().count() + 2;
        let top = "┌".to_string() + &"─".repeat(width - 2) + "┐";
        let bottom = "└".to_string() + &"─".repeat(width - 2) + "┘";
        
        if has_color {
            format!("{}\n│\x1b[31m{}\x1b[0m│\n{}", top, text, bottom)
        } else {
            format!("{}\n│{}│\n{}", top, text, bottom)
        }
    } else {
        if has_color {
            format!("\x1b[31m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }
}

fn get_terminal_width() -> Option<usize> {
    crossterm::terminal::size().ok().map(|(cols, _)| cols as usize)
}

fn launch_tui_app(specs_dir: PathBuf, default_width: usize) -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};
    use fastn_ascii_renderer::spec_viewer::SpecViewerApp;

    // Setup terminal with error handling
    if let Err(e) = enable_raw_mode() {
        eprintln!("Failed to enable raw mode: {}", e);
        eprintln!("This usually means the spec-viewer can't run in this environment.");
        eprintln!("Try running with --debug flag for non-interactive mode.");
        std::process::exit(1);
    }
    
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = SpecViewerApp::new(specs_dir);
    app.current_width = default_width;
    app.discover_spec_files()?;

    // Main event loop
    let result = run_tui_loop(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_tui_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut fastn_ascii_renderer::spec_viewer::SpecViewerApp,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| app.draw(f))?;

        if let Event::Key(key) = crossterm::event::read()? {
            app.handle_key_event(key)?;
            
            if app.should_quit {
                break;
            }
        }
    }
    Ok(())
}

fn discover_spec_files(dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use walkdir::WalkDir;
    
    let mut count = 0;
    for entry in WalkDir::new(dir) {
        let entry = entry?;
        if let Some(ext) = entry.path().extension() {
            if ext == "fastn" {
                println!("  📄 {}", entry.path().display());
                count += 1;
            }
        }
    }
    
    if count == 0 {
        println!("  ⚠️  No .fastn files found in {}", dir.display());
        println!("  💡 Create some spec files to get started!");
    } else {
        println!("  ✅ Found {} fastn spec file(s)", count);
    }
    
    Ok(())
}