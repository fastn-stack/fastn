# Specification File Format Design

## Strict Format Requirements

### **Check Mode Behavior: STRICT**
The check mode enforces **exact formatting** to ensure consistency and predictable parsing.

#### **Required File Structure:**
```
# 40x64

[Plain ASCII]          [ANSI Version]



# 80x128

[Plain ASCII]          [ANSI Version]



# 120x192

[Plain ASCII]          [ANSI Version]



```

### **Strict Formatting Rules:**

#### **1. Dimension Headers**
```
# 40x64
↑ ↑  ↑  ↑
│ │  │  └─ No trailing spaces
│ │  └─ No space before 'x'  
│ └─ Exactly one space after #
└─ Must start with # (no indentation)
```

**Valid:**
- `# 40x64`
- `# 80x128`
- `# 120x192`

**Invalid:**
- `#40x64` (missing space after #)
- `# 40 x 64` (spaces around x)
- `  # 40x64` (indentation)
- `# 40x64 ` (trailing space)

#### **2. Section Spacing**
```
# 40x64
[blank line]
[content starts...]
[content ends...]
[blank line]
[blank line]  
[blank line]
[blank line]
# 80x128
```

**Strict Requirements:**
- **Exactly 1 blank line** after dimension header
- **Exactly 4 blank lines** before next dimension header
- **No trailing whitespace** on blank lines
- **Consistent throughout file**

#### **3. Side-by-Side Format**
```
╭────────╮          ╭────────╮
│ Content│          │[31mContent[0m│
╰────────╯          ╰────────╯
↑        ↑         ↑
│        │         └─ ANSI version starts here
│        └─ Exactly 10 spaces separation  
└─ Plain ASCII version (no ANSI codes)
```

**Spacing Requirements:**
- **Exactly 10 spaces** between plain and ANSI versions
- **Consistent padding** to align plain version width
- **No mixed tabs and spaces**

#### **4. Content Alignment**
```
# Plain version must be padded to consistent width within each line
╭────────╮          ╭────────╮    ← Both align perfectly
│ Short  │          │ Short  │  
│ Longer │          │ Longer │    ← Padding maintains alignment
╰────────╯          ╰────────╯
```

## Autofix Mode Behavior: LIBERAL

### **Liberal Parsing Philosophy**
Autofix mode **accepts broken/inconsistent formats** and regenerates with perfect strict formatting.

#### **Accepted Input Variations:**
```
# Broken spacing - ACCEPTED
#40x64
# 80 x 128  
  #120x192

# Inconsistent content - ACCEPTED  
╭─broken─╮
│missing │
(incomplete output)

# Missing dimensions - ACCEPTED
# 40x64
(only one dimension present)

# Mixed formatting - ACCEPTED
Some plain text without proper formatting
Random ANSI codes: [31mred[0m
Inconsistent spacing
```

#### **Autofix Regeneration Process:**
1. **Ignore existing content** - Don't try to parse broken output
2. **Generate fresh** - Create clean output from component definition
3. **Apply strict format** - Use exact spacing and formatting rules
4. **Include all dimensions** - Always generate 40×64, 80×128, 120×192
5. **Perfect side-by-side** - Proper alignment and spacing

### **Format Validation Examples**

#### **Valid Format (Check Mode Passes):**
```
# 40x64

╭────────────────────────╮          ╭────────────────────────╮
│      Hello World       │          │      Hello World       │
╰────────────────────────╯          ╰────────────────────────╯



# 80x128

╭───────────────────────────────────╮          ╭───────────────────────────────────╮
│             Hello World           │          │             Hello World           │
╰───────────────────────────────────╯          ╰───────────────────────────────────╯



# 120x192

╭─────────────────────────────────────────────╮          ╭─────────────────────────────────────────────╮
│                  Hello World                │          │                  Hello World                │
╰─────────────────────────────────────────────╯          ╰─────────────────────────────────────────────╯



```

#### **Invalid Format (Check Mode Fails, Autofix Accepts):**
```
#40x64
broken content


# 80 x 128 

Some random text without proper structure

Missing 120x192 completely
```

## Implementation Strategy

### **Check Mode (Strict Validation):**
```rust
fn validate_format_strict(content: &str) -> ValidationResult {
    // Check exact header format: "# {width}x{height}"
    let header_regex = Regex::new(r"^# \d+x\d+$").unwrap();
    
    // Check exact spacing requirements
    let sections = content.split("# ").skip(1); // Skip empty first split
    
    for section in sections {
        // Validate header format
        let lines: Vec<&str> = section.lines().collect();
        let header = lines.get(0).ok_or("Missing header")?;
        
        if !header_regex.is_match(header) {
            return Err(format!("Invalid header format: '{}'", header));
        }
        
        // Check spacing after header (exactly 1 blank line)
        if lines.get(1) != Some(&"") {
            return Err("Header must be followed by exactly one blank line");
        }
        
        // Check spacing before next section (exactly 4 blank lines at end)
        let content_end = lines.len().saturating_sub(4);
        for i in content_end..lines.len() {
            if lines.get(i) != Some(&"") {
                return Err("Must end with exactly 4 blank lines");
            }
        }
        
        // Validate side-by-side format (10 spaces separation)
        // ... detailed validation logic
    }
    
    Ok(())
}
```

### **Autofix Mode (Liberal Regeneration):**
```rust
fn autofix_liberal(file_path: &Path, component_name: &str) -> Result<String, Error> {
    // Completely ignore existing content - generate fresh
    let fresh_content = generate_all_dimensions(component_name)?;
    
    // Apply strict formatting rules
    format_strictly(fresh_content)
}
```

## Benefits of Strict/Liberal Strategy

### **Development Workflow:**
1. **Developer edits component** - May break formatting while experimenting
2. **Autofix regenerates** - Always creates perfect strict format
3. **Check validates** - Ensures specs meet exact standards
4. **CI integration** - Strict validation prevents format drift

### **Quality Assurance:**
- **Predictable parsing** - Strict format enables reliable tooling
- **Consistent appearance** - All specs follow identical formatting  
- **Developer friendly** - Autofix handles formatting burden
- **Maintainable** - Clear rules prevent format confusion

This design ensures **specification quality** while providing **developer convenience** through automated formatting.