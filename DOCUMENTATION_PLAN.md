# fastn.com Documentation & Specification Plan

## Overview
Transform fastn.com into the comprehensive hub for all fastn development, specifications, and design decisions. **Primary focus: Complete comprehensive fastn 0.4 documentation before moving to v0.5 content.** This ensures the current stable release is thoroughly documented for developers and the community.

## Current State Analysis

### Existing Documentation Structure
```
fastn.com/
├── ftd/                    # FTD Language docs (needs major updates)
├── docs/                   # General documentation  
├── get-started/           # Onboarding (needs review)
├── examples/              # Code examples (expand)
├── best-practices/        # Development practices (expand)  
├── tutorial/              # Learning materials (update)
└── book/                  # fastn book (comprehensive review needed)
```

### Issues Identified
1. **Outdated Content**: Many .ftd files reference old syntax/features
2. **Incomplete Coverage**: Missing comprehensive language specification
3. **Scattered Information**: Design decisions not centralized
4. **Limited Examples**: Need more practical, real-world examples
5. **Missing v0.5 Content**: No documentation of new architecture

## Proposed New Structure

### 1. Add v0.5 Development Hub
```
fastn.com/
├── v0.5/                           # NEW: v0.5 development documentation
│   ├── architecture/               # System architecture documents
│   │   ├── compiler-pipeline.ftd   # fastn-section → fastn-unresolved → fastn-resolved → fastn-compiler flow
│   │   ├── rendering-engine.ftd    # fastn-runtime architecture
│   │   ├── terminal-rendering.ftd  # Terminal rendering design & specs
│   │   ├── css-semantics.ftd       # CSS-like property system
│   │   └── continuation-system.ftd # fastn-continuation architecture
│   ├── design-decisions/           # Major design choices and rationale
│   │   ├── ssl-design.ftd          # SSL/TLS integration design
│   │   ├── automerge-integration.ftd # Automerge design decisions
│   │   ├── p2p-architecture.ftd    # P2P networking design
│   │   └── breaking-changes.ftd    # v0.4 → v0.5 breaking changes
│   ├── implementation-status/      # Current development status
│   │   ├── compiler-status.ftd     # What's implemented in compiler
│   │   ├── runtime-status.ftd      # Runtime implementation status
│   │   └── roadmap.ftd            # Development roadmap
│   └── specifications/             # Technical specifications
│       ├── terminal-rendering-spec.ftd # Comprehensive terminal rendering spec
│       ├── css-property-mapping.ftd    # CSS property to terminal mapping
│       └── component-behavior.ftd      # Component behavior specifications
```

### 2. Complete FTD 0.4 Language Specification (PRIORITY)
```
fastn.com/
├── language-spec/                  # NEW: Comprehensive language specification for fastn 0.4
│   ├── index.ftd                  # Language overview
│   ├── syntax/                    # Syntax specification
│   │   ├── sections.ftd           # Section syntax rules
│   │   ├── headers.ftd            # Header syntax and semantics
│   │   ├── comments.ftd           # Comment syntax
│   │   └── grammar.ftd            # Complete BNF grammar
│   ├── type-system/               # Type system specification
│   │   ├── primitive-types.ftd    # boolean, integer, decimal, string
│   │   ├── derived-types.ftd      # ftd.color, ftd.length, etc.
│   │   ├── records.ftd            # Record type definitions
│   │   ├── or-types.ftd           # Or-type definitions
│   │   └── type-inference.ftd     # Type inference rules
│   ├── components/                # Component system
│   │   ├── definition.ftd         # Component definition syntax
│   │   ├── invocation.ftd         # Component invocation rules
│   │   ├── arguments.ftd          # Argument passing semantics
│   │   ├── children.ftd           # Children handling
│   │   └── inheritance.ftd        # Property inheritance rules
│   ├── variables/                 # Variable system
│   │   ├── declaration.ftd        # Variable declaration rules
│   │   ├── scoping.ftd           # Scoping rules
│   │   ├── mutability.ftd        # Mutable vs immutable
│   │   └── references.ftd        # Variable references
│   ├── functions/                 # Function system
│   │   ├── definition.ftd         # Function definition
│   │   ├── calls.ftd             # Function calls
│   │   ├── expressions.ftd       # Expression evaluation
│   │   └── built-ins.ftd         # Built-in functions
│   └── modules/                   # Module system
│       ├── imports.ftd           # Import semantics
│       ├── exports.ftd           # Export rules
│       ├── aliases.ftd           # Alias system
│       └── package-system.ftd    # Package management
```

### 3. Enhanced Component Documentation
```
fastn.com/ftd/
├── kernel-components/              # ENHANCED: Comprehensive kernel docs
│   ├── text.ftd                   # Enhanced with more examples
│   ├── column.ftd                 # Layout behavior, CSS mapping
│   ├── row.ftd                    # Flexbox semantics
│   ├── container.ftd              # Box model behavior
│   ├── image.ftd                  # Media handling
│   ├── video.ftd                  # Video component
│   ├── audio.ftd                  # NEW: Audio component docs
│   ├── checkbox.ftd               # Form controls
│   ├── text-input.ftd             # Input handling
│   ├── iframe.ftd                 # Embedded content
│   ├── code.ftd                   # Code display
│   ├── rive.ftd                   # Animation support
│   ├── document.ftd               # Document root
│   ├── desktop.ftd                # Device-specific rendering
│   └── mobile.ftd                 # Mobile-specific behavior
├── terminal-rendering/             # NEW: Terminal-specific documentation
│   ├── overview.ftd               # Terminal rendering principles
│   ├── ascii-art-layouts.ftd      # ASCII box-drawing specifications
│   ├── ansi-color-support.ftd     # Color handling in terminals
│   ├── responsive-terminal.ftd    # Adapting to terminal width
│   └── interactive-elements.ftd   # Terminal interaction patterns
```

### 4. Comprehensive Examples & Tutorials
```
fastn.com/
├── examples/                       # EXPANDED: Real-world examples
│   ├── basic/                     # Simple component usage
│   ├── layouts/                   # Layout patterns
│   ├── forms/                     # Form building
│   ├── interactive/               # Interactive components
│   ├── responsive/                # Responsive design
│   ├── terminal-apps/             # NEW: Terminal application examples
│   └── full-applications/         # Complete application examples
├── cookbook/                       # NEW: Common patterns and solutions
│   ├── component-patterns/        # Reusable component patterns
│   ├── layout-recipes/            # Common layout solutions
│   ├── styling-techniques/        # Advanced styling
│   └── performance-tips/          # Optimization techniques
```

## Implementation Phases

**Priority Order: Complete fastn 0.4 documentation first, then move to v0.5**

### Phase 1: fastn 0.4 Documentation Foundation (Week 1-2)
1. **Audit existing content** - Identify outdated information in current fastn.com
2. **Write documentation standards** - Establish style guide for contributions
3. **Begin comprehensive FTD 0.4 language specification** - Start with syntax and type system
4. **Update existing kernel component docs** - Fix outdated examples and add missing content

### Phase 2: Complete fastn 0.4 Language Specification (Week 3-4)
1. **Finish comprehensive language specification** - Complete FTD 0.4 docs with full coverage
2. **Enhanced component documentation** - Add missing components (audio), improve existing ones
3. **Create comprehensive examples library** - Real-world usage patterns
4. **Write cookbook entries** - Common patterns and solutions for 0.4

### Phase 3: fastn 0.4 Polish & Community Ready (Week 5-6)
1. **Review and update all 0.4 content** - Ensure consistency and accuracy
2. **Add interactive examples** - Live code examples where possible
3. **Create learning paths** - Guided learning sequences for fastn 0.4
4. **Community contribution guides** - How to contribute to fastn documentation

### Phase 4: Begin v0.5 Documentation (Week 7-8)
1. **Create v0.5 directory structure** - Set up new documentation hierarchy
2. **Architecture documentation** - Document v0.5 architecture decisions
3. **Design decision documentation** - SSL, P2P, Automerge, terminal rendering design docs
4. **Create v0.5 specifications** - Terminal rendering, CSS mapping, component behavior specs

## Content Standards

### Documentation Quality Standards
1. **Complete Examples** - Every feature must have working examples
2. **ASCII Output Specs** - Terminal rendering must show expected output
3. **Cross-References** - Comprehensive linking between related concepts
4. **Version Compatibility** - Clear indication of version requirements
5. **Testing Instructions** - How to test/verify examples

### Code Example Standards
```ftd
-- ds.rendered: Example Title
  -- ds.rendered.input:
  
  \-- ftd.text: Hello World
  color: red
  
  -- ds.rendered.output:
  
    -- ftd.text: Hello World
    color: red
    
  -- end: ds.rendered.output

-- end: ds.rendered

-- ds.terminal-output: Terminal Rendering

```
Hello World  (in red)
```

-- end: ds.terminal-output
```

### File Organization Standards
- `.ftd` extension for all documentation
- Clear hierarchical structure
- Consistent naming conventions
- Index files for each directory
- Cross-reference files for navigation

## Success Metrics

1. **Completeness** - 100% coverage of language features
2. **Accuracy** - All examples tested against v0.5 codebase
3. **Usability** - Clear navigation and discoverability
4. **Community Adoption** - External contributions and feedback
5. **Developer Productivity** - Faster onboarding and development

## Next Steps

1. **Create v0.5 directory structure**
2. **Begin content audit of existing documentation**
3. **Start with language specification foundation**
4. **Set up testing infrastructure for documentation examples**
5. **Establish review process for new content**

This plan transforms fastn.com into the definitive resource for fastn development, ensuring that design decisions are documented, specifications are comprehensive, and developers have the resources they need to be productive.