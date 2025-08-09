# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is fastn 0.5, a complete rewrite of the fastn language focusing on compatibility with previous versions. fastn is a full-stack web development framework with its own language (.ftd files).

## Build Commands

```bash
# Build the entire workspace
cargo build

# Build with release optimizations
cargo build --release

# Run tests across all workspace members
cargo test

# Run clippy for linting
cargo clippy

# Format code
cargo fmt

# Run the fastn CLI
cargo run --bin fastn -- [commands]

# Common fastn commands
cargo run --bin fastn -- serve    # Start development server
cargo run --bin fastn -- build    # Build the project
cargo run --bin fastn -- render   # Render pages
```

## Architecture

### Workspace Structure

The project uses a Rust workspace with multiple crates:

- **fastn**: Main CLI binary and command processing (src/main.rs)
- **fastn-compiler**: Core compiler implementation for FTD language
- **fastn-section**: Section parsing and AST handling
- **fastn-unresolved**: Handles unresolved symbols and forward references
- **fastn-package**: Package management and module system
- **fastn-router**: URL routing and request handling
- **fastn-wasm**: WebAssembly integration with HTTP, database (SQLite/PostgreSQL), and other services
- **fastn-continuation**: Async continuation provider system
- **fastn-utils**: Shared utilities and test helpers
- **fastn-static**: Static file serving
- **fastn-update**: Update management

### Key Concepts

1. **FTD Language**: The domain-specific language with `.ftd` extension
   - Components, functions, imports, and module system
   - Section-based syntax with headers and bodies

2. **Compilation Pipeline**:
   - Section parsing (fastn-section) → Unresolved AST (fastn-unresolved) → Resolution → Compilation (fastn-compiler)
   - Uses arena allocators for efficient memory management
   - Symbol resolution with dependency tracking

3. **Provider Pattern**: Uses continuation providers for async operations and data loading

4. **WASM Integration**: Supports WebAssembly for running compiled code with access to:
   - HTTP client/server operations
   - Database access (SQLite bundled, PostgreSQL optional)
   - Cryptography, AWS services, email

### File Patterns

- `.ftd` files: fastn template/component files
- Test files in `t/` directories (e.g., fastn-compiler/t/)
- Grammar definition: `fastn-compiler/grammar.bnf`

## Language Changes in v0.5

Key syntax changes from previous versions:
- Block section headers deprecated in favor of brace syntax
- Function arguments no longer need repetition in definition
- See README.md for detailed compatibility notes

## Database Configuration

- SQLite is bundled by default (see rusqlite configuration in Cargo.toml)
- PostgreSQL support is optional via the "postgres" feature flag in fastn-wasm

## Development Notes

- The project uses Rust edition 2024 with minimum version 1.86
- Uses `fastn-observer` for observability
- Async runtime: Tokio with multi-threaded runtime
- Avoid overly specific dependency versions (use "1" instead of "1.1.42" when possible)