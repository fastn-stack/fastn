# Check Mode Diff Output Design

## Enhanced Diff Display with Syntax Highlighting

### **Current Simple Output:**
```
❌ All dimensions: FAIL
   Snapshots differ from current rendering
```

### **Enhanced Diff Output:**
```
❌ All dimensions: FAIL

📝 Expected (specs/text/basic.rendered):
┌─ Expected ─────────────────────────────────────────────┐
│ # 40x64                                                │
│                                                        │
│ ╭────────────────╮          ╭────────────────╮         │
│ │  Hello World   │          │  Hello World   │         │
│ ╰────────────────╯          ╰────────────────╯         │
│                                                        │
│                                                        │
│                                                        │
│                                                        │
│ # 80x128                                               │
│                                                        │
│ ╭─────────────────────────────╮   ╭─────────────────────────────╮ │
│ │        Hello World          │   │        Hello World          │ │
│ ╰─────────────────────────────╯   ╰─────────────────────────────╯ │
└────────────────────────────────────────────────────────────────┘

🔧 Actual (current rendering):
┌─ Generated ────────────────────────────────────────────┐
│ # 40x64                                                │
│                                                        │
│ ╭──────────────────╮          ╭──────────────────╮     │
│ │   Hello World    │          │   Hello World    │     │
│ ╰──────────────────╯          ╰──────────────────╯     │
│                                                        │
│                                                        │
│                                                        │
│                                                        │
│ # 80x128                                               │
│                                                        │
│ ╭───────────────────────────────╮ ╭───────────────────────────────╮ │
│ │         Hello World           │ │         Hello World           │ │
│ ╰───────────────────────────────╯ ╰───────────────────────────────╯ │
└────────────────────────────────────────────────────────────────┘

🔍 Diff (- Expected, + Actual):
┌─ Changes ──────────────────────────────────────────────┐
│ # 40x64                                                │
│                                                        │
│- ╭────────────────╮          ╭────────────────╮        │
│- │  Hello World   │          │  Hello World   │        │
│- ╰────────────────╯          ╰────────────────╯        │
│+ ╭──────────────────╮          ╭──────────────────╮    │
│+ │   Hello World    │          │   Hello World    │    │
│+ ╰──────────────────╯          ╰──────────────────╯    │
│                                                        │
│ # 80x128                                               │
│                                                        │
│- ╭─────────────────────────────╮   ╭─────────────────────────────╮ │
│- │        Hello World          │   │        Hello World          │ │
│- ╰─────────────────────────────╯   ╰─────────────────────────────╯ │
│+ ╭───────────────────────────────╮ ╭───────────────────────────────╮│
│+ │         Hello World           │ │         Hello World           ││
│+ ╰───────────────────────────────╯ ╰───────────────────────────────╯│
└────────────────────────────────────────────────────────────────┘

💡 Summary: Border widths differ (padding changed)
   Expected: 16-char borders at 40ch, 29-char borders at 80ch  
   Actual:   18-char borders at 40ch, 31-char borders at 80ch

🔧 To accept changes: fastn-spec-viewer --autofix text/basic.ftd
```

## Syntax Highlighting Strategy

### **Color Coding:**
```rust
// Terminal color scheme for diff output
struct DiffColors {
    removed: Color::Red,           // Lines that should be removed (-)
    added: Color::Green,           // Lines that should be added (+)
    context: Color::White,         // Unchanged context lines
    header: Color::Blue,           // Section headers (# 40x64)
    border: Color::Cyan,           // Box drawing characters
    ansi_codes: Color::Yellow,     // ANSI escape sequences
}
```

### **Highlighting Rules:**

#### **1. Diff Line Prefixes:**
```
- Expected line    [RED background]
+ Actual line      [GREEN background]  
  Context line     [normal]
```

#### **2. Component Elements:**
```rust
// Syntax highlighting patterns
let patterns = [
    (r"^# \d+x\d+$", Color::Blue),           // Dimension headers
    (r"[╭╮╯╰┌┐┘└─│]", Color::Cyan),          // Box drawing
    (r"\x1b\[\d+m", Color::Yellow),         // ANSI codes
    (r"Hello World", Color::Magenta),       // Content text
];
```

#### **3. Side-by-Side Alignment:**
```
Plain Version                    ANSI Version
╭────────────────╮          ╭────────────────╮
│  Hello World   │          │  [31mHello World[0m   │
╰────────────────╯          ╰────────────────╯
↑                           ↑
└─ Highlighted differently  └─ ANSI codes highlighted
```

## Diff Implementation Strategy

### **1. Generate Comparison Files:**
```rust
fn check_with_diff(spec_file: &Path, expected: &str, actual: &str) -> DiffResult {
    // Create temporary files for diff
    let expected_file = write_temp_file("expected", expected)?;
    let actual_file = write_temp_file("actual", actual)?;
    
    // Generate structured diff
    let diff = generate_syntax_highlighted_diff(&expected_file, &actual_file)?;
    
    DiffResult {
        has_differences: expected != actual,
        diff_output: diff,
        summary: analyze_differences(expected, actual),
    }
}
```

### **2. Smart Diff Analysis:**
```rust
fn analyze_differences(expected: &str, actual: &str) -> DiffSummary {
    let mut summary = Vec::new();
    
    // Check header format differences
    if let Some(header_diff) = check_header_format_diff(expected, actual) {
        summary.push(format!("Header format: {}", header_diff));
    }
    
    // Check spacing differences  
    if let Some(spacing_diff) = check_spacing_diff(expected, actual) {
        summary.push(format!("Spacing: {}", spacing_diff));
    }
    
    // Check content differences
    if let Some(content_diff) = check_content_diff(expected, actual) {
        summary.push(format!("Content: {}", content_diff));
    }
    
    // Check alignment differences
    if let Some(alignment_diff) = check_alignment_diff(expected, actual) {
        summary.push(format!("Alignment: {}", alignment_diff));
    }
    
    DiffSummary { issues: summary }
}
```

### **3. Interactive Diff Viewer:**
```
📊 Component: text/basic.ftd - FAILED

🔍 Issues Found:
  1. Border width differs between expected and actual
  2. Text alignment shifted by 1 character  
  3. ANSI color codes placement inconsistent

📖 Detailed Diff:
  [Press 'v' to view full diff]
  [Press 'h' to view side-by-side]  
  [Press 's' to view summary only]
  [Press 'f' to autofix this component]

⚡ Quick Actions:
  [f] Fix this component
  [a] Fix all components
  [n] Next failed component
  [q] Quit
```

## Benefits of Enhanced Diff Output

### **Developer Experience:**
- **Visual diff** - Clear understanding of what changed
- **Syntax highlighting** - Easy to spot different types of changes
- **Smart analysis** - Categorized summary of issue types
- **Actionable guidance** - Clear steps to resolve issues

### **Quality Assurance:**
- **Precise validation** - Exact formatting requirements enforced
- **Clear feedback** - Developers know exactly what's wrong
- **Efficient debugging** - Highlighted differences speed troubleshooting
- **Consistent standards** - Strict format prevents spec drift

### **CI/CD Integration:**
```bash
# In CI pipeline with enhanced reporting
fastn-spec-viewer --check --verbose
# → Detailed diff output for failed specs
# → Clear summary of all issues found
# → Actionable guidance for fixing problems
```

This enhanced diff system transforms the check mode from basic pass/fail into a **comprehensive debugging and quality assurance tool** for specification development.