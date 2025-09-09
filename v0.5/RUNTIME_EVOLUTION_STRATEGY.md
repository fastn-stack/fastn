# Runtime Evolution Strategy for fastn Rendering Tools

## Strategic Runtime Development Plan

Both `fastn spec-viewer` and `fastn render` will eventually have **full runtime support** with event handling, interactivity, and navigation. The key is **incremental development** with shared learnings.

## Evolution Phases

### **Phase 1: Static Foundation (Completed ✅)**
**Tool**: `fastn spec-viewer` (static rendering only)

**Achievements:**
- ASCII rendering with Taffy layout engine
- Multi-width testing and comparison  
- File-based development workflow
- Visual specification development

**No Runtime Features:**
- ❌ No event handling
- ❌ No state management  
- ❌ No interactive components
- ❌ No navigation or routing

### **Phase 2: Runtime for Spec Viewer (Next 4-6 weeks)**
**Goal**: Add interactive capabilities to component specifications

**Runtime Features to Add:**
```rust
// Interactive component testing
fastn spec-viewer tui specs/ --interactive

// Component specs with events
-- ftd.text: Click Me
$on-click$: toggle($show-details)
color: $show-details ? red : blue

-- ftd.text: Details
visible: $show-details
```

**Runtime Architecture:**
```rust
struct SpecViewerRuntime {
    // Event handling
    js_runtime: quickjs::Runtime,
    event_handlers: HashMap<ComponentId, EventHandler>,
    
    // State management  
    component_state: HashMap<ComponentId, ComponentState>,
    variables: HashMap<String, Value>,
    
    // Rendering integration
    ascii_renderer: AsciiRenderer,
    layout_engine: TaffyLayoutEngine,
}
```

**Benefits:**
- **Interactive specs** - Test component behavior, not just appearance
- **Event testing** - Verify click handlers, state changes work correctly
- **Animation specs** - Simple animations in terminal (color changes, text updates)
- **Runtime learning** - Develop runtime expertise with simple components

### **Phase 3: Full Application Browser (6-8 weeks after Phase 2)**  
**Tool**: `fastn render` (complete terminal browser)

**Will Leverage Phase 2 Learnings:**
- ✅ **Runtime integration** - Reuse interactive architecture from spec-viewer
- ✅ **Event handling** - Apply proven event system to full applications
- ✅ **State management** - Scale component state to application state
- ✅ **ASCII rendering** - Same visual output, different context

**Additional Features:**
```rust
struct TerminalBrowser {
    // Application context (beyond spec-viewer)
    package_manager: PackageManager,
    database: Option<Database>,
    network_context: NetworkContext,
    url_router: UrlRouter,
    
    // Terminal browser UI
    address_bar: AddressBar,    // Shows id52 hostnames, URLs
    navigation: BrowserNav,     // Back/forward/reload
    
    // Shared with spec-viewer
    runtime: SpecViewerRuntime, // Same runtime system!
    renderer: AsciiRenderer,    // Same rendering system!
}
```

## Shared Runtime Architecture

### **Runtime Components (Shared Between Both Tools):**

#### **1. JavaScript/WASM Execution**
```rust
// Will be shared by both tools
pub struct FastnRuntime {
    js_engine: quickjs::Runtime,
    wasm_engine: Option<wasmtime::Engine>, // Future WASM support
}

impl FastnRuntime {
    fn execute_event_handler(&mut self, event: Event, component: &Component) -> StateChanges;
    fn update_variables(&mut self, updates: &[VariableUpdate]);
    fn evaluate_expressions(&self, expr: &Expression) -> Value;
}
```

#### **2. State Management**
```rust
// Shared state management system
pub struct StateManager {
    variables: HashMap<String, Variable>,
    component_states: HashMap<ComponentId, ComponentState>,
    change_listeners: Vec<StateChangeListener>,
}
```

#### **3. Event System**
```rust
// Shared event handling
pub enum UIEvent {
    Click { component: ComponentId, position: Position },
    KeyPress { key: Key, component: Option<ComponentId> },
    Resize { new_size: Size },
    Navigate { url: String }, // Only for fastn render
}
```

## Development Benefits

### **Incremental Learning:**
1. **Static rendering mastery** ✅ - Foundation solid
2. **Component runtime** 🚧 - Add interactivity to simple specs
3. **Application runtime** 🚧 - Scale to full applications with context

### **Risk Mitigation:**
- **Working tool throughout** - Always have functional spec-viewer
- **Shared codebase** - Runtime work benefits both tools
- **Proven architecture** - Each phase validates the next

### **Code Reuse Benefits:**
- **70%+ shared runtime** - Event handling, state management, JS integration
- **100% shared rendering** - ASCII output, layout calculations  
- **Parallel development** - Can work on both tools simultaneously

## Usage Evolution

### **Current (Phase 1):**
```bash
# Static component development
fastn spec-viewer render button.ftd
fastn spec-viewer test specs/components/
```

### **Phase 2 (Interactive Components):**
```bash  
# Interactive component testing
fastn spec-viewer tui specs/ --interactive
# → Click buttons, test state changes in component specs
```

### **Phase 3 (Full Applications):**
```bash
# Complete terminal browser
fastn render myapp.local/dashboard
# → Navigate, interact, full application experience
```

## Strategic Advantage

This **incremental runtime approach** ensures:

1. **Always working tools** - Never lose functionality during development
2. **Shared learnings** - Runtime expertise transfers between tools  
3. **Parallel value** - Component specs AND applications get interactive features
4. **Reduced complexity** - Build runtime once, use in both contexts

The **spec-viewer runtime** becomes the **proving ground** for the runtime architecture that will power the full terminal browser, ensuring robust, tested runtime capabilities when we build `fastn render`.

This evolution strategy maximizes **development efficiency** while minimizing **implementation risk** through incremental value delivery.