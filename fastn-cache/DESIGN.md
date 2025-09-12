# fastn-cache: FTD Compilation Caching System

## Overview

fastn-cache is a high-performance caching system designed specifically for FTD (fastn Document) compilation and incremental builds. It provides intelligent caching that dramatically improves fastn serve and fastn build performance while maintaining correctness through sophisticated dependency tracking.

## Performance Goals

- **fastn serve**: 5+ seconds → 8-20ms per request (200-400x improvement)
- **fastn build**: Full rebuild → Incremental rebuild (only changed files)
- **Correctness**: Always serve correct content, never stale cache
- **Developer Experience**: Transparent caching that "just works"

## Core Principles

### 1. Safety First
**"Cache sharing errors cause extra work, never wrong content"**

- Cache misses are acceptable (slower but correct)
- Wrong content served is never acceptable
- When in doubt, recompile rather than serve stale content

### 2. Dependency Tracking
**"Track what affects what, invalidate correctly"**

- Every FTD file knows what it depends on
- Any dependency change invalidates affected caches
- Includes packages, assets, configuration changes

### 3. Multi-Project Safety
**"Different projects must not interfere"**

- Each project gets isolated cache space
- Multiple clones of same project can share cache efficiently
- Test packages within repos get separate caches

## Architecture

### Cache Types

#### 1. FTD Parse Cache
**Purpose**: Cache parsed FTD documents to avoid re-parsing unchanged files

**Cache Key**: `{repo-name}+{relative-path}+{package-name}`

**Cache Content**:
```rust
struct ParseCache {
    hash: String,                    // Content + dependency hash
    dependencies: Vec<String>,       // File paths this document depends on  
    parsed_doc: ParsedDocument,      // Compiled FTD document
    created_at: SystemTime,          // Cache creation time
}
```

**Example**: 
- File: `~/fastn/examples/hello/FASTN.ftd`
- Cache Key: `fastn+examples_hello_FASTN.ftd+hello-world`
- Dependencies: `["FASTN.ftd", ".packages/doc-site/index.ftd", ...]`

#### 2. Incremental Build Cache
**Purpose**: Track which files need rebuilding based on changes

**Cache Content**:
```rust
struct BuildCache {
    documents: BTreeMap<String, DocumentMetadata>,
    file_checksums: BTreeMap<String, String>,
    packages_state: PackagesState,
    fastn_config_hash: String,
}

struct DocumentMetadata {
    html_checksum: String,     // Generated HTML hash
    dependencies: Vec<String>, // Files this document depends on
    last_built: SystemTime,    // When this was last built
}

struct PackagesState {
    packages_hash: String,     // Hash of .packages directory state
    last_updated: SystemTime,  // When packages were last updated
}
```

### Cache Directory Structure

```
~/.cache/
├── fastn+FASTN.ftd+fastn.com/           # fastn.com main project
├── fastn+examples_hello_FASTN.ftd+hello/   # hello example in fastn repo
├── fastn+test_basic_FASTN.ftd+test/        # test package in fastn repo  
├── my-blog+FASTN.ftd+my-blog/             # User's blog project
└── tutorial+FASTN.ftd+hello-world/        # Learning project
```

**Benefits**:
- Multiple test packages in same repo get isolated caches
- Different users' clones of same repo share cache efficiently  
- Clear, human-readable cache organization

### Dependency Tracking

#### File Dependencies
Every FTD file tracks its dependencies during compilation:

```rust
// Collected during import resolution
dependencies_during_render: Vec<String>

// Example for index.ftd:
[
    "FASTN.ftd",
    "components/hero.ftd", 
    ".packages/doc-site.fifthtry.site/index.ftd",
    ".packages/site-banner.fifthtry.site/banner.ftd"
]
```

#### Package Dependencies  
Track external package state for fastn update resilience:

```rust
// Include in dependency hash:
- .packages/{package}/last-modified-time
- FASTN.ftd content hash
- Package configuration changes
```

#### Dependency Invalidation
Cache is invalidated when ANY dependency changes:

```rust
fn is_cache_valid(cache_entry: &ParseCache) -> bool {
    let current_hash = generate_dependency_hash(
        &main_content,
        &cache_entry.dependencies  
    );
    current_hash == cache_entry.hash
}
```

## API Design

### Public Interface

```rust
pub struct FtdCache {
    config: CacheConfig,
    storage: CacheStorage,
}

pub struct CacheConfig {
    pub enabled: bool,
    pub cache_dir: Option<PathBuf>,
    pub max_cache_size: Option<u64>,
}

impl FtdCache {
    /// Create new cache instance for a fastn project
    pub fn new(config: CacheConfig) -> Result<Self>;
    
    /// Parse FTD file with caching
    pub fn parse_cached(
        &mut self, 
        file_id: &str, 
        source: &str,
        line_number: usize
    ) -> Result<ParsedDocument>;
    
    /// Update cache with collected dependencies after compilation
    pub fn update_dependencies(
        &mut self,
        file_id: &str, 
        dependencies: &[String],
        parsed_doc: &ParsedDocument
    ) -> Result<()>;
    
    /// Check if build is needed for incremental builds
    pub fn is_build_needed(&self, doc_id: &str) -> bool;
    
    /// Mark document as built with metadata
    pub fn mark_built(
        &mut self,
        doc_id: &str,
        html_checksum: &str,
        dependencies: &[String]
    ) -> Result<()>;
    
    /// Clear all cache (for troubleshooting)
    pub fn clear_all(&mut self) -> Result<()>;
    
    /// Get cache statistics for debugging
    pub fn stats(&self) -> CacheStats;
}

pub struct CacheStats {
    pub total_entries: usize,
    pub cache_size_bytes: u64,
    pub hit_rate: f64,
    pub last_cleanup: SystemTime,
}
```

### Internal Modules

```rust
mod storage;     // Disk I/O operations
mod keys;        // Cache key generation  
mod dependency;  // Dependency tracking
mod invalidation; // Cache invalidation logic
mod build;       // Build-specific caching
```

## Integration Points

### fastn-core Changes
```rust
// Remove from fastn-core:
- All caching utilities (utils.rs)
- cached_parse logic (doc.rs)  
- build cache module (build.rs)

// Add to fastn-core:
use fastn_cache::FtdCache;

// Replace caching calls:
let cache = FtdCache::new(config.cache_config())?;
let doc = cache.parse_cached(id, source, line_number)?;
```

### Configuration Integration
```rust
// In fastn main.rs:
let cache_config = CacheConfig {
    enabled: enable_cache_flag,
    cache_dir: None, // Use default
    max_cache_size: None, // Unlimited
};
```

## Use Cases Handled

### Development Workflow
1. **File edit** → Dependency tracking detects change → Cache invalidated → Recompile
2. **Import new file** → Dependencies updated → Cache includes new dependency
3. **Package update** (fastn update) → .packages state change → All caches invalidated

### Build Workflow  
1. **Initial build** → Parse all files → Cache metadata with dependencies
2. **File change** → Check dependencies → Rebuild only affected files
3. **Clean build** → Clear cache → Full rebuild

### Multi-Project Safety
1. **Project A** builds → Caches in `fastn+FASTN.ftd+project-a/`
2. **Project B** builds → Caches in `fastn+FASTN.ftd+project-b/`
3. **No interference** → Each project isolated

### Learning/Testing
1. **fastn/test1/** → Cache: `fastn+test1_FASTN.ftd+test/`
2. **fastn/test2/** → Cache: `fastn+test2_FASTN.ftd+test/`
3. **Isolation** → Tests don't affect each other

## Migration Strategy

### Phase 1: Create fastn-cache crate
- Extract storage utilities
- Create clean API
- Add comprehensive tests

### Phase 2: Migrate FTD parse caching
- Move cached_parse to fastn-cache
- Update fastn-core to use new API
- Verify performance maintained

### Phase 3: Migrate incremental build
- Move build cache system
- Update build command integration
- Test incremental build functionality

### Phase 4: Cleanup
- Remove old caching code from fastn-core
- Update documentation
- Performance verification

## Success Metrics

### Performance
- fastn serve: Sub-50ms response times with cache enabled
- fastn build: >90% reduction in rebuild time for incremental changes
- Cache hit rate: >95% for unchanged content

### Correctness
- Zero stale content incidents
- Automatic invalidation on any relevant file change
- Resilient to fastn update, configuration changes

### Developer Experience
- Transparent operation (no manual cache management)
- Clear error messages for cache issues
- Easy debugging with cache statistics

## Future Enhancements

### Advanced Features
- Cache size limits with LRU eviction
- Distributed cache for CI/CD systems
- Cache warming for faster cold starts
- Cache compression for space efficiency

### Monitoring
- Cache hit/miss metrics
- Performance tracking
- Cache corruption detection and auto-repair

---

This design provides the foundation for a world-class FTD caching system that prioritizes correctness while delivering massive performance improvements.