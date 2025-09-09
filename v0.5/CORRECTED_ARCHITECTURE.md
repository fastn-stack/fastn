# Corrected fastn Rendering Architecture - Key Learnings

## Critical Architectural Discoveries

### **❌ Original Mistakes:**
1. **Terminology confusion** - Called fastn files "components" instead of "documents"
2. **Responsibility mixing** - Put spec management in rendering engine  
3. **Manual calculations** - Bypassed CSS engine with hardcoded math
4. **API pollution** - Renderer knew about spec folder structure

### **✅ Corrected Architecture:**

#### **fastn-ansi-renderer (Pure Rendering Engine)**
**Single Responsibility:** Render fastn documents to ANSI terminal output

```rust
// Clean API - only knows about documents and rendering
pub struct DocumentRenderer;

impl DocumentRenderer {
    /// Core API - render any fastn document  
    pub fn render_from_source(source: &str, width: usize, height: usize) -> Result<Rendered, Error>;
    pub fn render_document(doc: &FastnDocument, width: usize, height: usize) -> Result<Rendered, Error>;
}

/// Structured output with multiple format options
pub struct Rendered {
    ansi_output: String,
}

impl Rendered {
    pub fn to_ansi(&self) -> &str;        // Terminal display with colors
    pub fn to_plain(&self) -> String;     // Editor viewing without escape codes  
    pub fn to_side_by_side(&self) -> String; // Spec file format
}
```

**Responsibilities:**
- ✅ **CSS layout** - Taffy integration for accurate layout calculations
- ✅ **ANSI rendering** - Terminal graphics with Unicode and colors  
- ✅ **Document parsing** - Simple fastn document structure
- ❌ **NO spec knowledge** - Doesn't know about embedded specs or folder structure

#### **fastn-spec-viewer (Specification Management + UI)**
**Single Responsibility:** Manage component specifications and provide browsing UI

```rust
// Embedded specification management
const EMBEDDED_SPECS: &[(&str, &str)] = &[
    ("text/basic.ftd", "-- ftd.text: Hello World"),
    ("text/with-border.ftd", "-- ftd.text: Hello World\nborder-width.px: 1\npadding.px: 8"),
    // ... all official fastn component specifications
];

// UI and workflow management - calls renderer for actual rendering  
let document_source = get_embedded_spec("text/with-border.ftd");
let rendered = fastn_ansi_renderer::DocumentRenderer::render_from_source(&document_source, 80, 128)?;

// Choose appropriate format for context
println!("{}", rendered.to_ansi());        // Terminal preview
save_to_file(rendered.to_side_by_side());  // Spec file
```

**Responsibilities:**
- ✅ **Embedded specs** - Official fastn component examples  
- ✅ **UI workflows** - TUI browser, file management, check/autofix
- ✅ **Format handling** - Choose ANSI vs plain vs side-by-side as needed
- ❌ **NO rendering logic** - Pure consumer of DocumentRenderer API

## Terminology Corrections

### **Proper fastn Terminology:**

#### **Document (.ftd file)**
```ftd
-- ftd.text: Hello World        ← This is a document containing components
border-width.px: 1

-- ftd.column:                  ← Multiple components in one document  
    -- ftd.text: Child 1        ← Nested components
    -- ftd.text: Child 2
-- end: ftd.column
```

#### **Component (within document)**
- **ftd.text** - Text component with properties
- **ftd.column** - Layout component with children
- **ftd.button** - Interactive component

#### **CSS Properties (not manual calculations)**
- **border-width.px: 1** → Taffy CSS border property
- **padding.px: 8** → Taffy CSS padding property  
- **width: fill-container** → Taffy CSS width: 100%

## API Design Principles

### **Clean Separation of Concerns:**

#### **1. Rendering Layer (fastn-ansi-renderer)**
```rust
// Pure function - document in, structured output out
let rendered = DocumentRenderer::render_from_source(
    "-- ftd.text: Hello\nborder-width.px: 1", 
    80, 
    128
)?;
```

#### **2. Format Layer (Rendered type)**  
```rust
// Consumer chooses appropriate format
match use_case {
    UseCase::Terminal => println!("{}", rendered.to_ansi()),
    UseCase::Editor => save_file(rendered.to_plain()),
    UseCase::SpecFile => save_file(rendered.to_side_by_side()),
}
```

#### **3. Application Layer (fastn-spec-viewer)**
```rust
// Manages specs, calls renderer, handles UI
for spec_name in embedded_specs {
    let source = get_embedded_spec(spec_name);
    let rendered = DocumentRenderer::render_from_source(&source, width, height)?;
    display_in_tui(rendered.to_ansi());
}
```

## Benefits of Corrected Architecture

### **Immediate Benefits:**
- ✅ **Clean API boundaries** - Each crate has single, clear purpose
- ✅ **Reusable renderer** - Any tool can render fastn documents  
- ✅ **Correct terminology** - Matches actual fastn language concepts
- ✅ **CSS accuracy** - Real CSS layout instead of manual approximations

### **Long-term Benefits:**
- ✅ **Maintainable** - Clear separation prevents architectural drift
- ✅ **Extensible** - Easy to add new format options or rendering backends
- ✅ **Testable** - Rendering logic isolated and independently verifiable  
- ✅ **Professional** - Architecture matches industry standards

### **Integration Ready:**
- **fastn render** - Can use same DocumentRenderer for terminal browser
- **fastn build** - Can use renderer for static site generation
- **Extensions** - Third-party tools can render fastn documents
- **GPU renderer** - Same document parsing, different rendering backend

## Key Learnings

### **1. Start with Correct Terminology:**
- **Documents** not "components" for .ftd files
- **Components** are elements within documents
- **CSS properties** not manual calculations

### **2. Proper Separation of Concerns:**
- **Rendering engine** - Pure function, no domain knowledge
- **Application layer** - Manages domain concepts, calls renderer
- **Format handling** - Structured output with consumer choice

### **3. API Design:**
- **Return structured types** not raw strings
- **Single responsibility** per crate
- **Clean boundaries** between layers

This corrected architecture provides the **professional foundation** needed for a production rendering system that can serve multiple use cases while maintaining clean, maintainable boundaries.