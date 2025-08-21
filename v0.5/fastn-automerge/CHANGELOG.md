# Changelog

All notable changes to the fastn-automerge crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Type-safe Path system** for document path validation
  - `fastn_automerge::Path` type prevents string-based path construction
  - `Path::from_string()` with validation - the only way to create paths from strings
  - `PathError` enum for structured path validation errors
  - Path validation enforces: non-empty paths, at most one '/-/' prefix
  - `rusqlite::ToSql` implementation for seamless database integration
  - `Display` trait for user-friendly path output

### Changed

- **Database API now uses typed paths**
  - All methods (`create`, `get`, `update`, `delete`, `exists`, `modify`, `get_document`) now accept `&Path` instead of `&str`
  - Compile-time path safety prevents typos and invalid paths
  - Breaking change: requires updating all path usage to use `Path::from_string()`

### Security

- **Path validation prevents malformed document paths**
  - Enforces consistent path structure across the application
  - Prevents empty paths and malformed '/-/' prefixes
  - Centralized validation logic for all path construction

## [0.1.0] - 2025-08-21

### Added

- Initial release of fastn-automerge crate
- Type-safe Rust API for Automerge CRDT documents with SQLite storage
- Complete CLI implementation with all CRUD operations
- Comprehensive test suite with fluent testing API
- Strict database lifecycle with separate init/open operations
- Actor ID management for multi-device scenarios
- Document history tracking with operation details
- Full integration with autosurgeon for type-safe serialization