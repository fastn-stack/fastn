use clap::Parser;
use std::path::PathBuf;

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

    println!("🚀 fastn spec-viewer starting...");
    println!("📁 Specs directory: {}", args.specs_dir.display());
    println!("📏 Default width: {} characters", args.width);
    
    if args.debug {
        println!("🔍 Debug mode enabled");
    }

    // TODO: Launch TUI application
    println!("🔧 TUI application coming soon...");
    println!();
    println!("For now, showing discovered spec files:");
    
    // Simple file discovery for Week 2
    discover_spec_files(&args.specs_dir)?;
    
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