use clap::Parser;
use std::path::PathBuf;
use crossterm::event::Event;
use ratatui::Terminal;

#[derive(Parser)]
#[command(name = "spec-viewer")]
#[command(about = "fastn UI component specification viewer")]
struct Cli {
    /// Path to specs directory
    #[arg(default_value = "specs")]
    specs_dir: PathBuf,
    
    /// Default preview width in characters
    #[arg(short, long, default_value = "80")]
    width: usize,
    
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    
    // Verify specs directory exists
    if !args.specs_dir.exists() {
        eprintln!("Specs directory '{}' not found!", args.specs_dir.display());
        eprintln!("Create it or specify a different path.");
        std::process::exit(1);
    }

    if args.debug {
        println!("🚀 fastn spec-viewer starting...");
        println!("📁 Specs directory: {}", args.specs_dir.display());
        println!("📏 Default width: {} characters", args.width);
        println!("🔍 Debug mode enabled");
        
        // Simple file discovery for debugging
        discover_spec_files(&args.specs_dir)?;
        return Ok(());
    }

    // Launch TUI application
    launch_tui_app(args.specs_dir, args.width)?;
    
    Ok(())
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