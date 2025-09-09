# fastn render - Terminal Browser Architecture

## Strategic Distinction

### **Two Different Rendering Contexts:**

#### **1. Spec Testing (`spec-viewer`)**
**Purpose**: Component specification development and testing
```bash
# Lightweight, single-file focused
cargo run --bin spec-viewer render text-card.ftd --width=80
```

**Characteristics:**
- ✅ **Single .ftd files** - No package complexity
- ✅ **Virtual package** - Framework creates minimal package context
- ✅ **Source visibility** - Shows folder structure, source code
- ✅ **Development tools** - Multi-width testing, comparison, generation
- ✅ **Fast iteration** - Minimal overhead for component development

#### **2. Full Application Rendering (`fastn render`)**  
**Purpose**: Complete fastn applications with packages, DB, dynamic content
```bash
# Full-featured application browser
fastn render --url=myapp.ftd --id52=myapp.local
```

**Characteristics:**
- ✅ **Complete packages** - Package management, dependencies
- ✅ **Dynamic content** - Database connections, dynamic URLs
- ✅ **Application context** - Real fastn applications
- ✅ **Terminal browser** - Shows URL, id52 hostname, navigation
- ✅ **Production rendering** - Full fastn runtime environment

## Architecture Comparison

### **Spec Viewer (Lightweight)**
```
.ftd file → Virtual Package → ASCII Renderer → Terminal Output
                           ↓
                    (No DB, no networking, no complex packages)
```

**UI Focus:**
```
┌─ File Tree ─────┬─ Source ──────┬─ Rendered Output ──┐
│ specs/          │ -- ftd.text:  │ ┌─────────────────┐ │
│ ├─ text.ftd     │ border: 1     │ │  Hello World    │ │
│ ├─ card.ftd     │ color: red    │ └─────────────────┘ │
│ └─ button.ftd   │               │                     │
└─────────────────┴───────────────┴─────────────────────┘
   Development        Code           Preview
```

### **fastn render (Full Browser)**
```
fastn Package → Full Runtime → ASCII Renderer → Terminal Browser
                  ↓
         (DB, networking, dynamic URLs, package deps)
```

**UI Focus:**
```
┌─ Terminal Browser ─────────────────────────────────────────────┐
│ myapp.local/dashboard/users?page=2               [◀] [▶] [⟳]  │
│                                                                │
│ ┌─ User Dashboard ────────────────────────────────────────────┐│
│ │                                                            ││
│ │  Welcome John! You have 5 new messages.                   ││
│ │                                                            ││
│ │  ┌─ Users ──────┬─ Messages ───┬─ Settings ──────────────┐││
│ │  │ • Alice      │ Hey there!   │ Theme: Dark             │││
│ │  │ • Bob        │ Meeting @3pm │ Language: English       │││
│ │  │ • Carol      │ Great work!  │ Notifications: On       │││
│ │  └──────────────┴──────────────┴─────────────────────────┘││
│ └────────────────────────────────────────────────────────────┘│
│                                                                │
│ [←→] Navigate  [R] Reload  [I] Inspect  [Q] Quit               │
└────────────────────────────────────────────────────────────────┘
```

## Implementation Strategy

### **Shared Foundation**
```rust
fastn-ascii-renderer/
├── layout/              # SHARED: Taffy integration, CSS mapping
├── canvas/             # SHARED: ANSI rendering, Unicode drawing  
├── components/         # SHARED: Component renderers
└── apps/              # Application-specific
    ├── spec_viewer/    # Spec development tools
    └── terminal_browser/ # Full application browser
```

### **Package Context Handling**

#### **Spec Viewer: Virtual Package**
```rust
impl SpecRenderer {
    fn create_virtual_package(ftd_file: &Path) -> VirtualPackage {
        VirtualPackage {
            name: "spec-test".to_string(),
            root_file: ftd_file.to_path_buf(),
            dependencies: vec![], // Minimal deps
            database: None,       // No DB
            networking: false,    // No network
            dynamic_urls: false,  // Static only
        }
    }
}
```

#### **fastn render: Full Package**
```rust
impl ApplicationRenderer {
    fn load_full_package(package_path: &Path) -> Result<FastnPackage, PackageError> {
        FastnPackage {
            name: package_name,
            dependencies: resolve_dependencies()?,
            database: setup_database()?,
            networking: enable_networking(),
            dynamic_urls: parse_url_patterns()?,
            // Full fastn runtime context
        }
    }
}
```

## CLI Interface Design

### **Spec Viewer Commands**
```bash
# Component specification development
fastn spec-viewer render component.ftd --width=80
fastn spec-viewer test specs/components/
fastn spec-viewer generate button.ftd --widths=40,80,120
fastn spec-viewer tui specs/
```

### **Application Render Commands**
```bash
# Full application rendering
fastn render --package=./myapp --url=/dashboard
fastn render --package=./myapp --url=/users/123 --id52=myapp.local
fastn render --package=./myapp --follow  # Responsive browser mode
fastn render --package=./myapp --interactive  # Full terminal browser
```

## Feature Matrix

| Feature | Spec Viewer | fastn render |
|---------|-------------|--------------|
| **Single .ftd files** | ✅ Primary use case | ✅ Supported |
| **Package management** | ❌ Virtual only | ✅ Full support |
| **Database integration** | ❌ No DB | ✅ SQLite/Postgres |
| **Dynamic URLs** | ❌ Static only | ✅ Full routing |
| **Source code view** | ✅ Development focus | ❌ Browser focus |
| **Multi-width testing** | ✅ Spec development | ❌ Single responsive |
| **File tree navigation** | ✅ Development workflow | ❌ URL navigation |
| **URL bar & navigation** | ❌ File-based | ✅ Browser-like |
| **id52 hostnames** | ❌ Local files | ✅ Network identity |

## Usage Scenarios

### **Spec Development Workflow (spec-viewer):**
```bash
# Creating new component specifications  
mkdir specs/forms/
echo "-- ftd.checkbox: Remember me" > specs/forms/checkbox.ftd
fastn spec-viewer render specs/forms/checkbox.ftd --width=80
fastn spec-viewer generate specs/forms/checkbox.ftd
```

### **Application Development Workflow (fastn render):**
```bash
# Testing complete applications
fastn init myapp
cd myapp/
fastn render --url=/ --follow
# Resize terminal → See responsive app behavior

# Testing with different data
fastn render --url=/users --id52=myapp.local
# Full terminal browser experience
```

### **Quality Assurance Workflows:**

#### **Component QA (spec-viewer):**
```bash
# Test all component specs
fastn spec-viewer test specs/ --verbose
# Validates visual specifications

# Review visual changes
fastn spec-viewer tui specs/
# Interactive review of component changes
```

#### **Application QA (fastn render):**
```bash  
# Test application at different screen sizes
fastn render --package=./app --url=/dashboard --follow
# Resize terminal to test responsive behavior

# Test different routes
fastn render --package=./app --url=/users/123
fastn render --package=./app --url=/settings  
# Verify all application routes render correctly
```

## Implementation Benefits

### **Clear Separation of Concerns:**
- **Spec viewer** - Fast, lightweight, development-focused
- **fastn render** - Complete, production-like, application-focused

### **Optimal Developer Experience:**
- **Component developers** use spec-viewer for fast iteration
- **Application developers** use fastn render for full testing
- **Both tools** share the same ASCII renderer foundation

### **Scalable Architecture:**
- **Shared rendering core** - Same layout and ASCII output
- **Different contexts** - Minimal vs full fastn runtime
- **Feature flags** - Control inclusion in different fastn builds

This distinction ensures we have the **right tool for each job** - lightweight spec development with spec-viewer, full application testing with fastn render, both sharing the robust ASCII rendering foundation we've built.