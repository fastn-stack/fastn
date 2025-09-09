use clap::Parser;
use std::path::PathBuf;
use crossterm::event::Event;
use ratatui::Terminal;

#[derive(Parser)]
#[command(name = "spec-viewer")]
#[command(about = "fastn UI component specification viewer and testing tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser)]
enum Commands {
    /// Launch interactive TUI (default)
    Tui {
        /// Specs directory to browse
        #[arg(default_value = "specs")]
        directory: PathBuf,
        
        /// Default preview width
        #[arg(short, long, default_value = "80")]  
        width: usize,
        
        /// Debug mode (non-interactive)
        #[arg(short, long)]
        debug: bool,
    },
    
    /// Render single file to stdout
    Render {
        /// fastn file to render
        file: PathBuf,
        
        /// Width in characters
        #[arg(short, long)]
        width: Option<usize>,
        
        /// Auto-detect terminal width
        #[arg(long)]
        auto_width: bool,
        
        /// Follow terminal resize for responsive testing
        #[arg(short, long)]
        follow: bool,
        
        /// Save output to .rendered file
        #[arg(short, long)]
        save: bool,
    },
    
    /// Test files against expected renders
    Test {
        /// File or directory to test
        target: PathBuf,
        
        /// Test specific widths only (e.g., "40,80,120")
        #[arg(long)]
        widths: Option<String>,
        
        /// Show detailed differences
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Generate .rendered files
    Generate {
        /// fastn files to generate renders for
        files: Vec<PathBuf>,
        
        /// Widths to generate (default: 40,80,120)  
        #[arg(long, default_value = "40,80,120")]
        widths: String,
        
        /// Force overwrite existing files
        #[arg(short, long)]
        force: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command.unwrap_or(Commands::Tui {
        directory: PathBuf::from("specs"),
        width: 80,
        debug: false,
    }) {
        Commands::Tui { directory, width, debug } => {
            handle_tui_mode(directory, width, debug)
        },
        Commands::Render { file, width, auto_width, follow, save } => {
            handle_render_mode(file, width, auto_width, follow, save)
        },
        Commands::Test { target, widths, verbose } => {
            handle_test_mode(target, widths, verbose)
        },
        Commands::Generate { files, widths, force } => {
            handle_generate_mode(files, widths, force)
        },
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