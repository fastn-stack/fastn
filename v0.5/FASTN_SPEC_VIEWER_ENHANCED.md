# fastn spec-viewer Enhanced Design

## Enhanced Preview Panel Architecture

### **Multi-Width Testing with Stored Comparisons**

The preview panel becomes a **comprehensive testing environment** with multiple modes and comparison capabilities.

## Preview Panel Modes

### **1. Pre-rendered Comparison Mode (Default)**

**File Discovery Pattern:**
```
specs/basic/text-border.fastn           # Main spec file
specs/basic/text-border.rendered-40     # 40-char width expected output  
specs/basic/text-border.rendered-80     # 80-char width expected output
specs/basic/text-border.rendered-120    # 120-char width expected output
```

**Panel Layout:**
```
┌─────────────────────── Preview: text-border.fastn ────────────────────────┐
│  Width: 80 chars  [◀ 40 | 80▶ | 120 ]                                    │
│                                                                            │
│  Expected (.rendered-80):          │  Current Render:                     │
│  ┌─────────────────┐               │  ┌─────────────────┐                 │
│  │                 │               │  │                 │                 │
│  │  Hello World    │               │  │  Hello World    │                 │
│  │                 │               │  │                 │                 │
│  └─────────────────┘               │  └─────────────────┘                 │
│                                    │                                      │
│  Status: ✅ MATCH                  │  [S] Save as rendered-80             │
│                                    │  [R] Regenerate all widths           │
└────────────────────────────────────────────────────────────────────────────┘
```

**When Differences Exist:**
```
│  Status: ❌ MISMATCH - Different border style                             │
│  Expected: ┌─┐  Current: ╔═╗                                              │
│  [S] Save Current  [D] Show Diff  [R] Regenerate                          │
```

### **2. Responsive Mode**

**Toggle with `[R]` - Uses actual terminal width**

```
┌─────────────────────── Responsive Mode ───────────────────────────────────┐
│  Terminal Width: 95 chars (live resize)                    [R] Fixed Mode │
│                                                                            │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │                                                                     │  │
│  │  Hello World - this text adapts to actual terminal width          │  │  
│  │                                                                     │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                            │
│  [S] Save as rendered-95  [F] Fullscreen  [T] Toggle panels               │
└────────────────────────────────────────────────────────────────────────────┘
```

### **3. Fullscreen Mode** 

**Toggle with `[F]` - Hide tree and source panels**

```
┌──────────────────────── Fullscreen Preview ──────────────────────────────┐
│  text-border.fastn @ 80 chars                              [F] Exit Full  │
│                                                                            │
│                                                                            │
│  ┌─────────────────┐                                                      │
│  │                 │                                                      │
│  │  Hello World    │                                                      │
│  │                 │                                                      │
│  └─────────────────┘                                                      │
│                                                                            │
│                                                                            │
│  ◀ Previous Spec    1/2/3: Width    Next Spec ▶                           │
│                                                                            │
│  [S] Save  [R] Responsive  [D] Diff  [Q] Quit                             │
└────────────────────────────────────────────────────────────────────────────┘
```

## Enhanced Keyboard Controls

### **Navigation:**
- `↑/↓` - Navigate file tree  
- `Enter` - Select file
- `◀/▶` - Switch between pre-rendered widths (40/80/120)
- `PgUp/PgDn` - Scroll long previews

### **Mode Switching:**
- `F` - Toggle fullscreen (hide tree + source panels)
- `R` - Toggle responsive mode (fixed width vs terminal width)
- `T` - Toggle tree panel (hide/show file tree)
- `H` - Help overlay

### **File Operations:**
- `S` - Save current render to .rendered-{width} file
- `Shift+S` - Save to all width variants (.rendered-40/80/120)
- `D` - Show diff between expected vs current
- `Ctrl+R` - Force regenerate current file

### **Testing:**
- `Space` - Run test verification for current file
- `Shift+Space` - Run tests for all files in current directory
- `V` - Validate all rendered files against current output

## File Structure & Workflow

### **Spec File Organization:**
```
specs/
├── components/
│   ├── text/
│   │   ├── basic.fastn
│   │   ├── basic.rendered-40
│   │   ├── basic.rendered-80
│   │   ├── basic.rendered-120
│   │   ├── with-border.fastn
│   │   ├── with-border.rendered-40
│   │   ├── with-border.rendered-80
│   │   └── with-border.rendered-120
│   └── layout/
│       ├── column-spacing.fastn
│       └── column-spacing.rendered-80    # Only 80-width if that's all we need
```

### **Development Workflow:**

#### **Creating New Specs:**
1. Create `new-spec.fastn` in external editor
2. Spec viewer auto-detects file, shows "No rendered files" 
3. View live preview, test at different widths
4. Press `S` to save current render as .rendered-{width}
5. Press `Shift+S` to generate all width variants

#### **Updating Specs:**
1. Edit `spec.fastn` in external editor  
2. Spec viewer auto-reloads, shows current vs expected
3. If different, preview shows ❌ MISMATCH status
4. Press `D` to see diff, `S` to update expected output

#### **Responsive Testing:**
1. Press `F` for fullscreen mode
2. Press `R` for responsive mode  
3. Resize terminal → Instant re-render
4. Test breakpoints and layout adaptation
5. Press `S` to save interesting responsive layouts

## Status Indicators

### **File Tree Status:**
```
specs/
├── components/
│   ├── text/
│   │   ├── ✅ basic.fastn              # All tests pass
│   │   ├── ❌ with-border.fastn        # Render mismatch  
│   │   ├── 🔄 typography.fastn        # Currently rendering
│   │   └── ⚠️  broken.fastn           # Parse error
```

### **Preview Panel Status:**
```
┌─ Status Bar ─────────────────────────────────────────┐
│ ✅ MATCH (3/3 widths)  │ Last updated: 2s ago       │
│ ❌ MISMATCH (1/3)      │ Auto-reload: ON             │  
│ 🔄 RENDERING...        │ File watcher: ACTIVE        │
│ ⚠️  PARSE ERROR        │ Terminal: 95x30             │
└──────────────────────────────────────────────────────┘
```

## Implementation Architecture

### **Enhanced App State:**
```rust
struct SpecViewerApp {
    // File management
    file_tree: FileTree,
    current_file: Option<PathBuf>,
    file_watcher: notify::RecommendedWatcher,
    
    // Preview state
    preview_mode: PreviewMode,
    current_width: usize,
    available_widths: Vec<usize>,  // [40, 80, 120]
    
    // Comparison state
    expected_renders: HashMap<usize, String>,  // width -> expected output
    current_render: Option<String>,
    render_status: RenderStatus,
    
    // UI state
    show_tree: bool,
    show_source: bool,
    fullscreen: bool,
}

enum PreviewMode {
    FixedWidth(usize),     # Show specific width (40/80/120)
    Responsive,            # Use actual terminal width  
    Comparison,            # Side-by-side expected vs current
}

enum RenderStatus {
    Match { width_count: usize },
    Mismatch { failed_widths: Vec<usize> },
    Rendering,
    ParseError(String),
    NoRenderedFiles,
}
```

### **Key Features Implementation:**

#### **Multi-Width File Loading:**
```rust
impl SpecViewerApp {
    fn load_rendered_files(&mut self, base_path: &Path) {
        self.expected_renders.clear();
        
        for width in &[40, 80, 120] {
            let rendered_path = base_path.with_extension(&format!("rendered-{}", width));
            if let Ok(content) = std::fs::read_to_string(&rendered_path) {
                self.expected_renders.insert(*width, content);
            }
        }
        
        self.available_widths = self.expected_renders.keys().copied().collect();
        self.available_widths.sort();
    }
    
    fn compare_renders(&self) -> RenderStatus {
        if let Some(current) = &self.current_render {
            let mut failed_widths = vec![];
            
            for (width, expected) in &self.expected_renders {
                let current_at_width = self.render_at_width(*width);
                if current_at_width.trim() != expected.trim() {
                    failed_widths.push(*width);
                }
            }
            
            if failed_widths.is_empty() {
                RenderStatus::Match { width_count: self.expected_renders.len() }
            } else {
                RenderStatus::Mismatch { failed_widths }
            }
        } else {
            RenderStatus::NoRenderedFiles
        }
    }
}
```

### **Enhanced Controls:**
```rust
impl SpecViewerApp {
    fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            // Width switching
            KeyCode::Left => self.previous_width(),
            KeyCode::Right => self.next_width(),
            KeyCode::Char('1') => self.set_width(40),
            KeyCode::Char('2') => self.set_width(80),
            KeyCode::Char('3') => self.set_width(120),
            
            // Mode switching  
            KeyCode::Char('f') | KeyCode::Char('F') => self.toggle_fullscreen(),
            KeyCode::Char('r') | KeyCode::Char('R') => self.toggle_responsive_mode(),
            KeyCode::Char('t') | KeyCode::Char('T') => self.toggle_tree_panel(),
            
            // File operations
            KeyCode::Char('s') => self.save_current_render(),
            KeyCode::Char('S') => self.save_all_widths(),
            KeyCode::Char('d') | KeyCode::Char('D') => self.show_diff_mode(),
            
            // Testing
            KeyCode::Char(' ') => self.run_test_current_file(),
            KeyCode::Char('v') | KeyCode::Char('V') => self.validate_all_renders(),
            
            _ => {}
        }
    }
}
```

## Visual Design Examples

### **Comparison Mode (Default):**
```
┌─ text-border.fastn @ 80 chars ────────── Status: ❌ MISMATCH ─────────────┐
│                                                                           │
│  Expected (.rendered-80):          │  Current Render:                    │
│  ┌─────────────────┐               │  ╔═════════════════╗                │ 
│  │                 │               │  ║                 ║                │
│  │  Hello World    │               │  ║  Hello World    ║                │
│  │                 │               │  ║                 ║                │
│  └─────────────────┘               │  ╚═════════════════╝                │
│                                    │                                     │
│  Last saved: 5 min ago             │  Rendered: just now                 │
│                                    │                                     │  
│ [S] Save Current [D] Diff [R] Regen │ [◀ 40 | 80 | 120 ▶] Width         │
└─────────────────────────────────────────────────────────────────────────────┘
```

### **Diff Mode:**
```
┌─ DIFF: Expected vs Current ─────────────────────────────────────────────────┐
│                                                                           │
│  Line 1: │ ┌─────────────────┐  │ ╔═════════════════╗                   │
│  Line 2: │ │                 │  │ ║                 ║                   │  
│  Line 3: │ │  Hello World    │  │ ║  Hello World    ║                   │
│  Line 4: │ │                 │  │ ║                 ║                   │
│  Line 5: │ └─────────────────┘  │ ╚═════════════════╝                   │
│                                                                           │
│  Changes: Border style (single → double)                                  │
│  [S] Accept Current  [ESC] Back to Preview                                │  
└─────────────────────────────────────────────────────────────────────────────┘
```

### **Fullscreen + Responsive Mode:**
```
┌──────────────────── text-border.fastn (Responsive) ───────────────────────┐
│  Terminal: 127x35 chars (resize me!)                       [F] Exit Full  │
│                                                                            │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │                                                                     │  │
│  │  Hello World - this text adapts to your terminal width perfectly  │  │
│  │                                                                     │  │  
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                            │
│                                                                            │
│                                                                            │
│  Resize your terminal to test responsive behavior!                        │
│  [S] Save as rendered-127  [1/2/3] Fixed widths  [Q] Quit                 │
└────────────────────────────────────────────────────────────────────────────┘
```

## Development Workflow Enhancement

### **Spec Creation Workflow:**
1. **Create** `.fastn` file in external editor
2. **Preview** in spec viewer - shows "No rendered files"
3. **Test** at different widths using ◀▶ or 1/2/3 keys
4. **Save** interesting widths with `S` (current) or `Shift+S` (all)
5. **Commit** both `.fastn` and `.rendered-*` files

### **Spec Update Workflow:**  
1. **Edit** `.fastn` file in external editor
2. **Auto-reload** shows current vs expected comparison
3. **Review** differences in comparison mode
4. **Use diff mode** (`D`) to see exact changes
5. **Update** expected files (`S`) or **fix** fastn file
6. **Verify** all width variants still pass

### **Regression Testing Workflow:**
1. **Navigate** to spec directory
2. **Press `V`** to validate all rendered files  
3. **Review** any mismatches in the status panel
4. **Use navigation** to check specific failures
5. **Batch update** or individual fixes as needed

## Implementation Details

### **File Pattern Recognition:**
```rust
struct SpecFile {
    fastn_file: PathBuf,                           // main.fastn
    rendered_files: HashMap<usize, PathBuf>,       // width -> rendered file
    last_modified: SystemTime,
}

impl SpecFile {
    fn discover_rendered_files(&mut self) {
        let base = self.fastn_file.with_extension("");
        for width in [40, 80, 120] {
            let rendered_path = PathBuf::from(format!("{}.rendered-{}", base.display(), width));
            if rendered_path.exists() {
                self.rendered_files.insert(width, rendered_path);
            }
        }
    }
    
    fn needs_regeneration(&self) -> bool {
        let fastn_modified = self.fastn_file.metadata()?.modified()?;
        self.rendered_files.values().any(|rendered_file| {
            rendered_file.metadata()?.modified()? < fastn_modified
        })
    }
}
```

### **Responsive Rendering:**
```rust  
impl PreviewPanel {
    fn render_at_width(&self, width: usize) -> String {
        // Use our ASCII renderer with specified width
        let available_space = taffy::Size {
            width: taffy::AvailableSpace::Definite((width * 8) as f32), // chars → px  
            height: taffy::AvailableSpace::MaxContent,
        };
        
        self.ascii_renderer.render_with_constraints(&self.current_spec, available_space)
    }
    
    fn render_responsive(&self, terminal_size: (u16, u16)) -> String {
        let (width, height) = terminal_size;
        self.render_at_width(width as usize - 4) // Account for panel borders
    }
}
```

## Benefits of Enhanced Design

### **Development Efficiency:**
- **Instant feedback** - See changes immediately
- **Multi-width testing** - Test responsive behavior easily  
- **Regression prevention** - Saved comparisons catch unintended changes
- **Visual debugging** - See exactly what changed

### **Quality Assurance:**
- **Comprehensive testing** - All common widths automatically tested
- **Diff visualization** - Clear view of what changed
- **Batch validation** - Test entire spec suite quickly
- **Save/restore workflow** - Easy to update or revert expected outputs

### **Collaboration:**
- **Visual specs** - Non-developers can see component behavior
- **Clear differences** - Easy to review spec changes  
- **Terminal-native** - Works in any development environment
- **Self-contained** - No external dependencies or setups

This enhanced spec viewer becomes the **definitive tool for fastn component development**, providing comprehensive testing and validation capabilities through an intuitive visual interface.

The tool **dogfoods the ASCII renderer extensively**, ensuring we build a robust rendering engine through real-world usage.