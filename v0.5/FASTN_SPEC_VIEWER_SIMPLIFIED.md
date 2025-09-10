# Simplified fastn spec-viewer Design

## Focused Scope & Embedded Specs

### **Strategic Simplification:**

#### **Embedded Specification Browser**
- ✅ **Specs compiled into binary** - No external spec files needed
- ✅ **Self-contained** - Works immediately after fastn installation  
- ✅ **Curated content** - Only official fastn component specifications
- ✅ **Universal access** - Everyone can explore fastn UI capabilities

#### **Two Simple Usage Modes:**

### **1. Interactive Browser Mode (Default)**
```bash
fastn spec-viewer
```

**Full-screen TUI with embedded specs:**
```
┌─ fastn Component Specifications ─────────────────────────────────────────┐
│                                                                          │
│  📁 Components                          ┌─ Preview @ 80 chars ───────────┐ │
│    ├─ text/                             │                                │ │
│    │  ├─ basic                          │  ┌─────────────────┐            │ │
│    │  ├─ with-border         ◀─ Current │  │                 │            │ │
│    │  └─ typography                     │  │  Hello World    │            │ │
│    ├─ layout/                          │  │                 │            │ │
│    │  ├─ column-spacing                 │  └─────────────────┘            │ │
│    │  └─ row-layout                     │                                │ │
│    └─ forms/                           │  [1] 40ch [2] 80ch [3] 120ch    │ │
│       ├─ checkbox                      │  [R] Responsive [F] Fullscreen  │ │
│       └─ text-input                    └────────────────────────────────────┘ │
│                                                                          │
│  ↑↓: Navigate  Enter: Preview  1/2/3: Width  R: Responsive  Q: Quit      │
└──────────────────────────────────────────────────────────────────────────┘
```

**Features:**
- ✅ **Tree navigation** of embedded specs
- ✅ **Live preview** with width switching (40/80/120)
- ✅ **Responsive mode** adapts to terminal resize  
- ✅ **Fullscreen mode** for focused component viewing

### **2. Direct Render Mode**
```bash
# Render specific component to fullscreen
fastn spec-viewer text/with-border

# Render to stdout (for piping/redirecting)  
fastn spec-viewer text/with-border --stdout

# Custom width for stdout
fastn spec-viewer text/with-border --stdout --width=120
```

**Direct render behavior:**
- **No `--stdout`**: Fullscreen responsive preview
- **With `--stdout`**: Print to stdout (for automation/piping)
- **Width detection**: Auto-detect terminal or use `--width`

## Simplified CLI Interface

### **Command Structure:**
```rust
#[derive(Parser)]
#[command(name = "spec-viewer")]
#[command(about = "fastn component specification browser")]
struct Cli {
    /// Specific spec to view (e.g., "text/with-border", "layout/column")  
    spec_path: Option<String>,
    
    /// Output to stdout instead of fullscreen preview
    #[arg(long)]
    stdout: bool,
    
    /// Width for stdout output (auto-detects terminal if not specified)
    #[arg(short, long)]
    width: Option<usize>,
}
```

**Usage Examples:**
```bash
# Interactive browser (default)
fastn spec-viewer

# Fullscreen component preview  
fastn spec-viewer text/with-border
# Shows component in responsive fullscreen mode

# Stdout output
fastn spec-viewer text/with-border --stdout
# Prints ASCII to stdout at terminal width

fastn spec-viewer text/with-border --stdout --width=80  
# Prints ASCII to stdout at 80 characters

# Piping/automation
fastn spec-viewer button/primary --stdout > component.txt
fastn spec-viewer form/login --stdout --width=120 | less
```

## Embedded Specs Architecture

### **Compile-Time Spec Inclusion:**
```rust
// During build, embed all spec files into binary
const EMBEDDED_SPECS: &[(&str, &str)] = &[
    ("text/basic", include_str!("../specs/text/basic.ftd")),
    ("text/with-border", include_str!("../specs/text/with-border.ftd")),
    ("layout/column", include_str!("../specs/layout/column.ftd")),
    ("forms/checkbox", include_str!("../specs/forms/checkbox.ftd")),
    // ... all official component specs
];
```

### **Runtime Spec Discovery:**
```rust
pub struct EmbeddedSpecRegistry {
    specs: HashMap<String, String>,   // path -> content
    categories: HashMap<String, Vec<String>>, // category -> spec list
}

impl EmbeddedSpecRegistry {
    pub fn load_embedded() -> Self {
        let mut specs = HashMap::new();
        let mut categories = HashMap::new();
        
        for (path, content) in EMBEDDED_SPECS {
            specs.insert(path.to_string(), content.to_string());
            
            // Build category tree
            if let Some(category) = path.split('/').next() {
                categories.entry(category.to_string())
                    .or_insert_with(Vec::new)
                    .push(path.to_string());
            }
        }
        
        Self { specs, categories }
    }
    
    pub fn get_spec(&self, path: &str) -> Option<&String> {
        self.specs.get(path)
    }
    
    pub fn list_categories(&self) -> Vec<String> {
        self.categories.keys().cloned().collect()
    }
}
```

### **App State Simplified:**
```rust
pub struct SpecViewerApp {
    // Embedded content (no file I/O at runtime)
    registry: EmbeddedSpecRegistry,
    current_spec_path: Option<String>,
    
    // Preview state
    current_width: usize,
    responsive_mode: bool,
    fullscreen: bool,
    
    // Navigation state
    selected_category: usize,
    selected_spec: usize,
    should_quit: bool,
}
```

## Benefits of Simplified Design

### **User Experience:**
- ✅ **Zero setup** - Works immediately after installing fastn
- ✅ **Complete reference** - All component specs always available
- ✅ **Universal access** - Same specs for everyone
- ✅ **Offline capable** - No dependency on external files

### **Distribution:**
- ✅ **Self-contained binary** - Specs included in fastn installation
- ✅ **Version consistency** - Specs match exact fastn version
- ✅ **No file path issues** - Embedded specs always work
- ✅ **Reduced support burden** - No "spec files missing" issues

### **Development Workflow:**
```bash
# Quick component reference
fastn spec-viewer                     # Browse all specs
fastn spec-viewer text/with-border    # Preview specific component

# Automation/documentation  
fastn spec-viewer button/primary --stdout --width=80 > docs/button.txt
fastn spec-viewer layout/grid --stdout | pandoc -o grid-layout.pdf

# Terminal integration
tmux split-window "fastn spec-viewer form/login"
# Side-by-side development with live component preview
```

## Implementation Simplification

### **Removed Complexity:**
- ❌ No arbitrary file support
- ❌ No directory browsing of user files
- ❌ No file watching (embedded content)
- ❌ No generate/test commands (handled by fastn development tools)

### **Focused Features:**
- ✅ **Embedded spec browser** - Navigate official fastn components
- ✅ **Direct component preview** - Quick fullscreen component viewing  
- ✅ **Stdout automation** - Integration with scripts and documentation
- ✅ **Responsive testing** - Terminal resize testing

This simplified design makes the spec-viewer **focused, reliable, and universally useful** - exactly what users need to explore and understand fastn component capabilities without any setup or configuration complexity.