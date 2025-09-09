# fastn spec-viewer Design & Product Requirements

## Product Overview

**fastn spec-viewer** - A terminal-based visual specification browser for fastn UI components. Enables real-time preview of fastn component rendering with responsive testing capabilities.

## Strategic Goals

### **Primary Use Cases:**
1. **Spec Development** - Visual feedback for fastn component specifications
2. **Responsive Testing** - Terminal resize testing for layout behavior
3. **Documentation** - Live examples of fastn component behavior  
4. **QA/Review** - Visual verification of component specs before merging

### **User Personas:**
- **fastn Developers** - Building components, need visual feedback
- **Spec Contributors** - Creating component specifications  
- **QA/Reviewers** - Verifying spec accuracy
- **End Users** - Exploring fastn component capabilities

## Product Requirements

### **Core Features**

#### 1. **File Browser (Left Panel)**
```
specs/
├── components/
│   ├── text/
│   │   ├── basic.fastn           ←─ Current file
│   │   ├── with-border.fastn
│   │   └── typography.fastn
│   ├── layout/
│   │   ├── column-spacing.fastn
│   │   └── row-layout.fastn
│   └── forms/
│       └── checkbox.fastn
```

**Features:**
- [x] **Tree navigation** - Arrow keys, Enter to select
- [x] **File filtering** - Show only `.fastn` files
- [x] **Status indicators** - ✅ Valid, ❌ Parse error, 🔄 Rendering
- [x] **Folder expand/collapse** - Space to toggle folders

#### 2. **Source View (Middle Panel)**
**Read-only display of selected fastn file**

```
-- ftd.text: Hello World        │
color: red                      │
border-width.px: 1              │
padding.px: 8                   │
                                │
-- ftd.column:                  │
spacing.fixed.px: 16            │
                                │
    -- ftd.text: Child 1        │
    -- ftd.text: Child 2        │
                                │
-- end: ftd.column              │
```

**Features:**
- [x] **Syntax highlighting** - fastn language syntax coloring
- [x] **Error highlighting** - Mark parse errors in red
- [x] **Line numbers** - Easy reference
- [x] **Auto-scroll** - Follow current component

#### 3. **Live Preview (Right Panel)**
**Real-time ASCII rendering with ANSI colors**

```
┌─────────────────┐
│                 │
│  Hello World    │  (red text)
│                 │
└─────────────────┘


Child 1


Child 2
```

**Features:**
- [x] **Live ANSI colors** - Real terminal color output
- [x] **Auto-refresh** - Updates on file changes
- [x] **Scrolling** - Large layouts scrollable
- [x] **Responsive** - Terminal resize triggers re-render

### **Advanced Features**

#### 4. **Multi-Width Testing**
- [x] **Width toggle** - Test at 40/80/120 character widths
- [x] **Responsive preview** - See layout adaptation
- [x] **Breakpoint testing** - Manual width adjustment
- [x] **Auto-generation** - Create .fastn-rendered-{width} files

#### 5. **Auto-Reload System**
- [x] **File watching** - Detect external .fastn file changes
- [x] **Instant preview** - Sub-second update latency  
- [x] **Error recovery** - Graceful handling of invalid files
- [x] **Status feedback** - Clear loading/error/success states

#### 6. **Export & Testing**
- [x] **Save test case** - Export current render to .fastn-rendered file
- [x] **Regression testing** - Compare with existing test cases
- [x] **Batch verification** - Test all specs in directory

## Technical Architecture

### **Dual CLI Strategy with Feature Flags**

#### **Development CLI** (Minimal, Fast)
```bash
# In fastn-ascii-renderer crate - always available
cargo run --bin spec-viewer
# Fast compilation, only ASCII renderer dependencies
# No feature flags, always builds for development
```

#### **End User CLI** (Feature-Gated Integration)
```bash
# In main fastn CLI - controlled by feature flag
fastn spec-viewer     # Only available if built with --features=spec-viewer
```

**Build Strategy:**
```toml
# Default fastn release (minimal)
cargo build --release
# fastn binary without spec-viewer (smaller size)

# Development/full fastn release  
cargo build --release --features=spec-viewer
# fastn binary with spec-viewer included
```

**Feature Flag Implementation:**
```toml
# In fastn/Cargo.toml
[features]
default = []
spec-viewer = ["fastn-ascii-renderer/spec-viewer"]

[dependencies]
fastn-ascii-renderer = { workspace = true, optional = true }

# In fastn-ascii-renderer/Cargo.toml  
[features]
default = []
spec-viewer = ["ratatui", "crossterm", "notify", "syntect", "walkdir"]

# TUI dependencies only when spec-viewer feature enabled
ratatui = { version = "0.28", optional = true }
crossterm = { version = "0.27", optional = true }
notify = { version = "6.1", optional = true }
syntect = { version = "5.1", optional = true }
walkdir = { version = "2.4", optional = true }
```

**Release Strategy Control:**
- **Minimal fastn** - No spec-viewer overhead for production deployments
- **Development fastn** - Full capabilities including spec-viewer
- **User choice** - Advanced users can build with spec-viewer if needed
- **Clean separation** - Core fastn functionality not affected by spec-viewer complexity

### **Implementation Strategy**

#### **Phase 1: Minimal CLI in ascii-renderer**
```rust
fastn-ascii-renderer/
├── src/
│   ├── bin/
│   │   └── spec_viewer.rs    # Standalone CLI for development
│   └── spec_viewer/          # Core functionality
│       ├── app.rs           # Main TUI application
│       ├── file_tree.rs     # File navigation widget
│       ├── source_view.rs   # Syntax highlighted source
│       ├── preview.rs       # ASCII render output
│       └── file_watcher.rs  # Auto-reload system
```

#### **Phase 2: fastn Integration**
```rust  
fastn/src/commands/
└── spec_viewer.rs           # Integration with main CLI
```

### **Technology Stack**

```toml
[dependencies]
# Core TUI
ratatui = "0.28"             # Terminal UI framework
crossterm = "0.27"           # Cross-platform terminal  

# File operations
notify = "6.1"               # File watching
walkdir = "2.4"              # Directory traversal

# Syntax highlighting  
syntect = "5.1"              # Code highlighting
once_cell = "1.19"           # Lazy static for syntax sets

# Existing dependencies
taffy = "0.5"                # Layout (already have)
ansi_term = "0.12"           # Colors (already have)
```

### **UI Layout (ratatui)**

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, Paragraph},
    style::{Color, Style},
};

// Three-panel layout
let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Length(30),    // File tree
        Constraint::Percentage(40), // Source view  
        Constraint::Percentage(60), // Preview
    ])
    .split(area);
```

## User Experience Flow

### **Typical Session:**
```bash
# Launch spec viewer
cargo run --bin spec-viewer

# Navigate to interesting spec
↓ ↓ Enter  # Navigate to basic text spec

# Open external editor  
# Edit spec file in VS Code/Vim

# Save file → Spec viewer instantly updates preview
# Resize terminal → Preview re-renders at new width
# Press 1/2/3 → Test at different widths

# Navigate to next spec
↓ Enter

# Repeat cycle
```

### **Power User Features:**
```bash
# Batch test all specs
T → Run test verification for entire directory

# Export current render  
S → Save current preview as .fastn-rendered file

# Multi-width testing
1 → 40 chars, 2 → 80 chars, 3 → 120 chars

# Debug mode
D → Show layout calculation details
```

## Implementation Benefits

### **Development Velocity:**
- **Visual feedback loop** - See changes instantly
- **No context switching** - Spec viewer + editor side-by-side
- **Responsive testing** - Terminal resize = instant layout test
- **Error visualization** - Parse errors immediately visible

### **Quality Assurance:**
- **Visual verification** - See exactly how specs render
- **Multi-width testing** - Catch responsive issues early
- **Regression prevention** - Compare with saved test cases  
- **ANSI validation** - See actual terminal colors

### **Dual CLI Benefits:**
- **Fast development** - Minimal compilation for daily use
- **End user access** - Full fastn integration for distribution
- **Feature flexibility** - Advanced features can go to main CLI
- **Maintenance simplicity** - Core logic shared, CLI layers separate

## Success Metrics

### **Developer Experience:**
- [ ] **Sub-second reload** - File change → preview update in <500ms
- [ ] **Zero-config setup** - Works out of box with any specs/ folder  
- [ ] **Keyboard efficiency** - All navigation via keyboard
- [ ] **Visual clarity** - Easy to see layout structure and colors

### **Tool Adoption:**
- [ ] **Daily development use** - Becomes essential for spec development
- [ ] **Contribution quality** - Better specs from contributors using tool
- [ ] **Testing efficiency** - Faster spec verification and debugging
- [ ] **End user value** - Users can explore fastn capabilities visually

This tool becomes the **definitive way to develop and verify fastn UI specifications**, providing immediate visual feedback and comprehensive testing capabilities.

The spec viewer will be our **first real application** built with the ASCII renderer - perfect dogfooding to validate the rendering architecture works correctly!