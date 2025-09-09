# fastn spec CLI Commands Design

## Git-Style Subcommand Design

### **1. `fastn spec` / `fastn spec render` - Component Browser & Renderer**

**Purpose**: Browse and preview fastn component specifications

```bash
# Interactive TUI browser (default)
fastn spec

# Direct component preview
fastn spec render text/with-border.ftd  
fastn spec render components/button.ftd

# Stdout output for automation  
fastn spec render text/basic.ftd --stdout
fastn spec render button.ftd --stdout --width=120 --height=192

# Backwards compatibility
fastn spec text/basic.ftd         # Same as: fastn spec render text/basic.ftd
fastn spec text/basic.ftd --stdout # Same as: fastn spec render text/basic.ftd --stdout
```

**Features**:
- Three-panel TUI (file tree + source + preview)
- Responsive preview with width switching (1/2/3 keys)
- Help system (? key) with complete feature documentation
- Stdout mode for automation and documentation generation

### **2. `fastn spec check` - Snapshot Validation**

**Purpose**: Validate component specifications against saved snapshots

```bash
# Check all specs in directory
fastn spec check

# Check specific component
fastn spec check text/with-border.ftd

# Check with verbose diff output  
fastn spec check --verbose

# Check only specific dimensions
fastn spec check --widths=40x64,80x128
```

**Features**:
- Discovers all .ftd files in specs/ directory
- Tests at multiple dimensions: 40x64, 80x128, 120x192
- Clear pass/fail reporting with diff visualization
- CI-friendly exit codes (0 = all pass, 1 = failures)
- Missing snapshot detection

### **3. `fastn spec fix` - Snapshot Auto-Update**

**Purpose**: Update/create snapshots for component specifications

```bash
# Auto-fix all failing/missing snapshots
fastn spec fix

# Fix specific component only
fastn spec fix text/with-border.ftd

# Fix specific dimensions
fastn spec fix --widths=80x128,120x192

# Preview changes before applying
fastn spec fix --dry-run
```

**Features**:
- Updates failing snapshots to match current rendering
- Creates missing .rendered-WxH files
- Batch operations for efficiency
- Dry-run mode to preview changes
- Selective fixing by component or dimensions

## Command Usage Examples

### **Development Workflow:**

#### **Creating New Specs:**
```bash
# 1. Create component spec
echo "-- ftd.text: New Component\nborder-width.px: 1" > specs/new-component.ftd

# 2. Preview in browser
fastn spec-viewer new-component.ftd

# 3. Create snapshots
fastn spec-fix new-component.ftd

# 4. Validate everything works
fastn spec-check new-component.ftd
```

#### **Updating Existing Specs:**
```bash
# 1. Edit spec file
vim specs/text/with-border.ftd

# 2. See what changed
fastn spec-check text/with-border.ftd --verbose

# 3. Update snapshots if changes are correct
fastn spec-fix text/with-border.ftd

# 4. Validate full suite still passes
fastn spec-check
```

### **CI/CD Integration:**
```bash
# In CI pipeline - validate no regressions
fastn spec-check || exit 1

# During development - fix issues automatically  
fastn spec-check || fastn spec-fix --dry-run
```

### **Documentation Generation:**
```bash
# Generate ASCII art for documentation
for component in text/basic text/with-border components/button; do
  echo "## $component" >> docs.md
  fastn spec-viewer $component --stdout --width=80 >> docs.md
done
```

## Implementation Strategy

### **Separate Binaries:**
```
fastn-spec-viewer/
├── src/
│   ├── bin/
│   │   ├── fastn_spec_viewer.rs    # Interactive browser + stdout
│   │   ├── fastn_spec_check.rs     # Validation tool  
│   │   └── fastn_spec_fix.rs       # Snapshot update tool
│   └── lib.rs                      # Shared rendering logic
```

### **fastn CLI Integration:**
```rust
// In fastn/src/commands/mod.rs
pub enum Cli {
    // ... existing commands
    SpecViewer(SpecViewerArgs),
    SpecCheck(SpecCheckArgs), 
    SpecFix(SpecFixArgs),
}
```

**Command mapping:**
- `fastn spec-viewer` → `fastn-spec-viewer` binary
- `fastn spec-check` → `fastn-spec-check` binary  
- `fastn spec-fix` → `fastn-spec-fix` binary

## Benefits of Separated Commands

### **Clear Responsibilities:**
- **spec-viewer** - Human interaction, visual development
- **spec-check** - Automated validation, CI integration
- **spec-fix** - Maintenance, snapshot management

### **Optimized User Experience:**
- **No flag confusion** - Each command does one thing well
- **Focused help** - Relevant options for each command
- **Better discoverability** - Tab completion shows `fastn spec-*` tools
- **Workflow clarity** - View → Check → Fix cycle

### **Technical Benefits:**
- **Smaller binaries** - Only include needed functionality
- **Faster compilation** - Focused dependencies per command
- **Easy testing** - Each command independently testable
- **Clear API** - Well-defined purpose boundaries

This design creates a **professional spec development toolkit** with three complementary tools that work together seamlessly while maintaining clear separation of concerns.