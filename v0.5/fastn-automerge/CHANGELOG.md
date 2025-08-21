# Changelog

All notable changes to the fastn-automerge crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Type-safe DocumentId system** - `fastn_automerge::DocumentId` with validation
- **CLI architecture improvements** - Database instance passed to commands  
- `DocumentIdError` enum for structured validation

### Changed

- **Breaking: Database API uses typed document IDs** - All methods accept `&DocumentId` instead of `&str`
- **Breaking: CLI uses `eyre::Result`** - Precise error types instead of global enum
- Document ID validation: non-empty, at most one '/-/' prefix

### Removed

- Global error enum mixing in CLI contexts
- Duplicate database wrapper functions

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