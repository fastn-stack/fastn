# fastn v0.5 Dual-Renderer Architecture Design

## Executive Summary

**Strategic Decision**: Build **TWO complementary renderers** for fastn v0.5:

1. **ASCII-first renderer** - Immediate value, learning platform, specification tool
2. **GPU renderer** - Future high-fidelity rendering with multi-format output

This approach manages complexity through **incremental value delivery** while building toward browser-scale capability.

## Scale Recognition

We're building:
- **Language Runtime** - FTD execution engine
- **Layout Engine** - CSS-like positioning with flexbox/grid
- **Rendering Pipeline** - GPU-accelerated graphics
- **Multi-format Export** - ASCII, PDF, SVG, raster 
- **Event System** - Interactive UI with JS/WASM
- **Terminal Runtime** - Non-browser execution environment

**This is browser-scale complexity** requiring methodical, incremental approach.

## Technology Stack Research

### Core Technologies (Based on 2024 Rust Ecosystem)

#### 1. Layout Engine: **Taffy** (✅ Proven Choice)
```rust
taffy = "0.5"
```
- **Used by**: Dioxus, Bevy UI, Blitz renderer
- **Capabilities**: CSS Flexbox, Grid, Block layout algorithms
- **Performance**: Competitive with Meta's Yoga (React Native)
- **API**: `TaffyTree` for node trees, `compute_layout_with_measure()`

#### 2. GPU Rendering: **WGPU + Vello** (✅ Modern Stack)
```rust  
wgpu = "22"        // Cross-platform GPU abstraction
vello = "0.3"      // 2D vector graphics (used by Xilem)
```
- **WGPU**: WebGPU standard, runs on Vulkan/Metal/DX12/OpenGL/WebGL
- **Vello**: High-performance 2D graphics, GPU compute-based
- **Used by**: Xilem, Blitz renderer

#### 3. Text Rendering: **Cosmic Text** (✅ Advanced Typography)
```rust
cosmic-text = "0.12"  // Advanced text shaping/layout
```
- **Used by**: System76's COSMIC DE, many Rust GUI projects
- **Capabilities**: Complex text shaping, ligatures, emoji, i18n
- **Integration**: Works with WGPU via `glyphon` crate

#### 4. JS Runtime: **QuickJS** (✅ Already in fastn)
```rust  
// Already in fastn v0.4 - reuse existing integration
```

### Proven Integration Examples

#### **Blitz Renderer (DioxusLabs)**
```
Stylo (CSS parsing) → Taffy (layout) → Vello + WGPU (rendering)
```
- **Status**: Pre-alpha but functional HTML/CSS renderer
- **Architecture**: Exactly what we need for fastn
- **Output**: High-fidelity GPU rendering

#### **Xilem (Linebender)**  
```
Reactive UI → Masonry (widgets) → Vello + WGPU (graphics)
```
- **Status**: Experimental but active development
- **Features**: Native performance, GPU acceleration
- **Integration**: Direct WGPU + layout integration

## Proposed fastn v0.5 Architecture

### High-Level Pipeline

```
FTD Source → fastn-compiler → Layout Tree → Render Commands → Multi-format Output
```

### Detailed Architecture

```rust
// Core rendering pipeline
struct FastnRenderer {
    layout_engine: taffy::TaffyTree,        // CSS layout calculations
    gpu_context: wgpu::Device,              // GPU resource management  
    vector_renderer: vello::Renderer,       // 2D vector graphics
    text_engine: cosmic_text::FontSystem,   // Text shaping/layout
    js_runtime: quickjs::Runtime,           // Event handling
}

impl FastnRenderer {
    // Static rendering (for specification/testing)
    fn render_static(&self, ftd_doc: &Document) -> RenderResult;
    
    // Multi-format output
    fn to_ascii(&self, result: &RenderResult) -> String;
    fn to_pdf(&self, result: &RenderResult) -> Vec<u8>;
    fn to_svg(&self, result: &RenderResult) -> String;
    fn to_png(&self, result: &RenderResult) -> Vec<u8>;
    
    // Interactive runtime (for terminal/desktop apps)
    fn run_interactive(&self, ftd_doc: &Document) -> Runtime;
}
```

## Dual-Renderer Strategy

### **Why ASCII-First Approach Wins**

#### ✅ **Complexity Management Benefits**
- **Minimal dependencies**: Just `taffy` for layout, no GPU stack
- **Fast iteration**: Character output, no graphics pipeline complexity
- **Learning platform**: Master layout concepts with simple output
- **Immediate testing**: CI-friendly, no GPU/driver dependencies

#### ✅ **Strategic Value of "Duplication"**
- **ASCII renderer**: Production terminal apps (immediate value)
- **GPU renderer**: Desktop/web apps (future value)
- **Shared foundation**: 70%+ code reuse in layout engine
- **Risk mitigation**: Two working renderers vs one complex renderer

#### ✅ **Code Sharing Architecture**
```rust
fastn-layout-engine/     // SHARED between both renderers
├── css_properties.rs    // FTD → CSS conversion
├── layout_tree.rs      // Component tree structure  
├── taffy_integration.rs // Taffy wrapper
└── component_traits.rs  // Renderer-agnostic interfaces

fastn-ascii-renderer/   // ASCII-specific implementation
├── canvas.rs          // ANSI character grid + colors
├── ascii_output.rs    // Character-based rendering
└── components/        // ASCII component renderers

fastn-gpu-renderer/     // GPU-specific implementation (future)
├── wgpu_context.rs    // GPU setup and management
├── vello_integration.rs // Vector graphics rendering
└── export/           // PDF, SVG, PNG output
```

## Revised Implementation Strategy

### **Phase 1: ASCII Renderer Foundation (4-6 weeks)**
**Goal**: Production-quality ASCII rendering with proper CSS layout

**Week 1-2: Layout Engine**
```rust
// Taffy integration with CSS property mapping
struct LayoutEngine {
    taffy: taffy::TaffyTree,
    css_mapper: FtdToCssMapper,
}

impl LayoutEngine {
    fn parse_ftd_component(&mut self, comp: &FtdComponent) -> taffy::NodeId;
    fn compute_layout(&mut self) -> LayoutResult;
    fn get_computed_style(&self, node: taffy::NodeId) -> ComputedStyle;
}
```

**Week 3-4: ASCII Canvas**  
```rust
// ANSI-capable character canvas
struct AsciiCanvas {
    grid: Vec<Vec<char>>,
    colors: Vec<Vec<AnsiColor>>,
    width: usize,
    height: usize,
}

impl AsciiCanvas {
    fn draw_rectangle(&mut self, rect: Rect, style: BoxStyle);
    fn draw_text(&mut self, pos: Position, text: &str, style: TextStyle);
    fn to_ansi_string(&self) -> String;  // With color codes
}
```

**Week 5-6: Component Integration**
```rust
// Render FTD components with proper layout
struct ComponentRenderer {
    layout_engine: LayoutEngine,
    canvas: AsciiCanvas,
}

impl ComponentRenderer {
    fn render_text(&mut self, text: &FtdText) -> RenderResult;
    fn render_column(&mut self, column: &FtdColumn) -> RenderResult;
    fn render_row(&mut self, row: &FtdRow) -> RenderResult;
}
```

**Deliverable**: Full FTD → ASCII rendering with ANSI colors, proper layout

### **Phase 2: GPU Renderer (Future - 8-10 weeks)**
**Goal**: High-fidelity rendering with multi-format export

**Approach**: Build on ASCII renderer learnings with shared layout engine

```rust
// Reuse layout engine from ASCII renderer
fastn-gpu-renderer/
├── layout/           // SHARED: Reuse from ASCII renderer  
├── wgpu_context/     // GPU setup, headless rendering
├── vello_renderer/   // Vector graphics rendering
├── cosmic_text/      // Advanced typography  
├── export/          // PDF, SVG, PNG generation
└── runtime/         // Interactive apps with QuickJS
```

**Dependencies**: 
- `taffy` (shared), `wgpu`, `vello`, `cosmic-text`, `quickjs`
- **Build on**: All layout logic from ASCII renderer

## Risk Mitigation

### **Complexity Management**
- **Break into 2-week phases** with working deliverables
- **Each phase stands alone** - can pause/pivot without losing work
- **Test-driven development** - Working tests before moving to next phase

### **Technical Risks**
- **WGPU headless rendering** - Research headless GPU context setup
- **ASCII conversion quality** - May need custom sampling algorithms  
- **Performance** - Profile early, optimize rendering pipeline
- **Integration complexity** - Keep components loosely coupled

### **Dependency Risks**
- **WGPU ecosystem stability** - Use stable versions, pin dependencies
- **Cosmic-text compatibility** - Verify WGPU integration works
- **Platform support** - Ensure headless rendering works on CI/servers

## Success Criteria

### **Phase Completion Metrics**
1. **Phase 1**: CSS layout calculations match expectations
2. **Phase 2**: GPU renders correct rectangles/borders  
3. **Phase 3**: Text rendering matches typography specifications
4. **Phase 4**: All output formats visually consistent
5. **Phase 5**: Interactive components respond correctly

### **Overall Goals**
- **CSS Layout Fidelity** - Matches web browser layout behavior
- **Multi-format Consistency** - Same visual output across ASCII/PDF/SVG  
- **Performance** - Fast enough for interactive terminal use
- **Specification Quality** - Clear visual examples for every FTD component

This design provides a **methodical path** from simple layout calculations to full browser-scale rendering, with clear milestones and fallback options if any phase proves too complex.

## Next Steps

1. **Validate Phase 1 approach** - Create minimal Taffy integration prototype
2. **Research headless WGPU** - Verify off-screen rendering capabilities  
3. **Design CSS property mapping** - FTD attributes → CSS properties
4. **Create integration test framework** - End-to-end verification system

This foundation will enable fastn v0.5 to have **production-quality rendering** with comprehensive output format support.