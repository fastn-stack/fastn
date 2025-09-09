# Enhanced fastn spec-viewer CLI Design

## Multi-Mode CLI Architecture

The spec-viewer supports **multiple usage patterns** for different development workflows:

### **1. Interactive TUI Mode (Default)**
**Full-featured visual development environment**
```bash
fastn spec-viewer                    # Launch TUI with current directory
fastn spec-viewer specs/components/  # Launch TUI with specific directory
```

### **2. Single File Render Mode**  
**Quick preview of individual components**
```bash
# Fixed width rendering
fastn spec-viewer render button.fastn --width=80

# Auto-detect terminal width
fastn spec-viewer render button.fastn --auto-width

# Follow terminal resize (responsive testing)
fastn spec-viewer render button.fastn --follow
```

### **3. Testing/Validation Mode**
**Automated testing and validation**
```bash
# Test single file
fastn spec-viewer test button.fastn

# Test entire directory  
fastn spec-viewer test specs/components/

# Generate missing .rendered files
fastn spec-viewer generate button.fastn --widths=40,80,120
```

## Command Line Interface

### **CLI Structure:**
```rust
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
    #[command(name = "tui")]
    Tui {
        /// Specs directory to browse
        #[arg(default_value = "specs")]
        directory: PathBuf,
        
        /// Default preview width
        #[arg(short, long, default_value = "80")]
        width: usize,
    },
    
    /// Render single file to stdout
    Render {
        /// fastn file to render
        file: PathBuf,
        
        /// Width in characters (overrides auto-detection)
        #[arg(short, long)]
        width: Option<usize>,
        
        /// Auto-detect terminal width  
        #[arg(long)]
        auto_width: bool,
        
        /// Follow mode: re-render on terminal resize
        #[arg(short, long)]
        follow: bool,
        
        /// Save output to .rendered-{width} file
        #[arg(short, long)]
        save: bool,
        
        /// Output format (ascii, ansi, plain)
        #[arg(long, default_value = "ansi")]
        format: String,
    },
    
    /// Test files against expected renders
    Test {
        /// File or directory to test
        target: PathBuf,
        
        /// Test only specific widths (comma-separated)
        #[arg(long)]
        widths: Option<String>,
        
        /// Verbose diff output
        #[arg(short, long)]
        verbose: bool,
        
        /// Update mismatched files automatically
        #[arg(short, long)]
        update: bool,
    },
    
    /// Generate .rendered files for components
    Generate {
        /// fastn file(s) to generate renders for
        files: Vec<PathBuf>,
        
        /// Widths to generate renders for
        #[arg(long, default_value = "40,80,120")]
        widths: String,
        
        /// Force overwrite existing files
        #[arg(short, long)]
        force: bool,
        
        /// Preview before saving  
        #[arg(short, long)]
        preview: bool,
    },
}
```

## Usage Scenarios

### **Quick Component Preview:**
```bash
# Basic rendering
$ fastn spec-viewer render text-card.fastn
Hello World

# With specific width
$ fastn spec-viewer render text-card.fastn --width=40  
┌──────────────────────────────────────┐
│                                      │
│  Hello World                         │
│                                      │
└──────────────────────────────────────┘

# Auto-width (detects terminal size)
$ fastn spec-viewer render text-card.fastn --auto-width
# Renders at current terminal width (e.g., 127 chars)

# Save the output
$ fastn spec-viewer render text-card.fastn --width=80 --save
# Creates text-card.rendered-80 with the output
```

### **Responsive Testing:**
```bash
$ fastn spec-viewer render responsive-layout.fastn --follow

# Terminal shows:
┌─ responsive-layout.fastn @ 95 chars (resize terminal to test) ─┐
│                                                                │
│  ┌─────────────┬─────────────┬─────────────┐                 │
│  │   Column 1  │   Column 2  │   Column 3  │                 │  
│  └─────────────┴─────────────┴─────────────┘                 │
│                                                                │
│  Press Ctrl+C to exit. Resize terminal to see layout adapt.   │
└────────────────────────────────────────────────────────────────┘

# User resizes terminal → Component re-renders immediately  
# Shows how layout adapts to different screen sizes
```

### **Development Workflow Integration:**
```bash
# Terminal 1: Edit component
$ vim specs/forms/checkbox.fastn

# Terminal 2: Live preview with follow mode
$ fastn spec-viewer render specs/forms/checkbox.fastn --follow

# Edit → Save → Instant preview update
# Resize → See responsive behavior  
# Perfect feedback loop for component development
```

### **Testing and Quality Assurance:**
```bash
# Test single component  
$ fastn spec-viewer test button.fastn
Testing button.fastn...
  ✅ 40ch: PASS (matches expected)
  ✅ 80ch: PASS (matches expected)  
  ❌ 120ch: FAIL (border differs)

Expected (120ch):
┌──────────────────────────┐
│                          │
│        Click Here        │
│                          │  
└──────────────────────────┘

Actual (120ch):
╔══════════════════════════╗
║                          ║
║        Click Here        ║
║                          ║
╚══════════════════════════╝

# Update the expected output
$ fastn spec-viewer test button.fastn --update
Updated: button.rendered-120

# Batch testing
$ fastn spec-viewer test specs/components/
Testing 15 files...
✅ 12 passed (36 width variants)
❌ 2 failed (6 width variants)  
⚠️  1 missing .rendered files

Failed:
  - card.fastn: 80ch, 120ch (border style changed)
  - modal.fastn: 40ch (text wrapping issue)

$ fastn spec-viewer test specs/components/ --update
Updated 3 .rendered files to match current output
```

## Benefits of Multi-Mode Design

### **Developer Productivity:**
- **Quick previews** - `render` for fast component checks
- **Responsive testing** - `--follow` for layout adaptation testing  
- **Batch validation** - `test` entire directories efficiently
- **Visual TUI** - Complex navigation and comparison

### **CI/CD Integration:**
```bash
# In CI pipeline
fastn spec-viewer test specs/ --verbose
# Fails build if any specs don't match expected output
# Perfect for preventing visual regressions
```

### **Documentation Generation:**
```bash  
# Generate all rendered examples
fastn spec-viewer generate specs/**/*.fastn --preview

# Review all outputs, then commit
git add specs/**/*.rendered-*
git commit -m "docs: update component visual specifications"
```

### **Team Workflow:**
```bash
# Designer creates new component
$ fastn spec-viewer generate new-card.fastn --preview
# Previews output, saves if satisfied

# Developer reviews spec  
$ fastn spec-viewer tui specs/cards/
# Visual review with comparison mode

# QA validates changes
$ fastn spec-viewer test specs/ --verbose  
# Comprehensive validation of all specs
```

## Implementation Priority

### **Phase 1: Render Mode (Week 3)**
- Single file rendering with width options
- Auto-width detection from terminal  
- Basic --save functionality

### **Phase 2: Follow Mode (Week 4)**  
- Terminal resize detection and re-rendering
- Real-time responsive testing
- Clean exit handling

### **Phase 3: Test Mode (Week 5)**
- Comparison with .rendered files
- Batch testing capabilities  
- Detailed diff output

### **Phase 4: Generate Mode (Week 6)**
- Automated .rendered file generation
- Preview before save
- Batch generation for multiple files

This multi-mode design makes the spec-viewer incredibly versatile - from quick command-line rendering to comprehensive visual development environment, covering every aspect of fastn component specification workflow.