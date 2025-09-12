# fastn-cache: FTD Compilation Caching System

## Journal

This section tracks day-by-day progress on caching implementation and testing for production confidence.

### 2025-09-11 - Initial Implementation Complete

**Performance Investigation:**
- ✅ Identified root cause: FTD compilation taking 5+ seconds per request
- ✅ Found disabled caching in `cached_parse()` function
- ✅ Re-enabled caching with `--enable-cache` flag
- ✅ Measured 200-400x performance improvement (5s → 8-20ms)

**Architecture Built:**
- ✅ Created fastn-cache crate with comprehensive DESIGN.md
- ✅ Implemented storage module with disk I/O operations
- ✅ Implemented dependency tracking with file change detection
- ✅ Fixed cross-project cache pollution bug (hardcoded "fastn.com" directory)
- ✅ Implemented git-aware cache key strategy for multi-project safety

**Incremental Build:**
- ✅ Fixed fastn build incremental system (uncommented dependency collection)
- ✅ Verified existing sophisticated incremental build infrastructure works
- ✅ Re-enabled dependency tracking that was disabled

**Testing Environment:**
- ✅ Branch: optimize-page-load-performance (14 commits)
- ✅ Compiles successfully: fastn-cache crate + fastn-core integration
- ✅ Performance verified: fastn serve with --enable-cache shows dramatic speedup
- ✅ Real projects tested: fastn.com (large), kulfi/malai.sh (medium)

**Current Status:**
- ✅ PR created: https://github.com/fastn-stack/fastn/pull/2199
- ✅ Design documented with 10 critical test scenarios
- ✅ Production safety measures defined
- ⏳ Test suite implementation needed for production confidence

**Next Steps:**
- Implement comprehensive test suite (10 scenarios)
- Verify cache correctness under all conditions
- Performance benchmarking and regression detection
- Real-world validation before production deployment

---

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

## Implementation Status

### ✅ Completed (Current State)
- **fastn-cache crate created**: Complete architecture with DESIGN.md
- **Storage module**: Disk I/O operations with corruption handling
- **Dependency tracking**: File change detection and transitive invalidation
- **Cache key strategy**: Git-aware, multi-project safe naming
- **fastn-core integration**: Dependency added and basic integration
- **--enable-cache flag**: Optional caching for production use
- **Incremental build fix**: Re-enabled existing dependency collection

### 🚧 In Progress  
- **Full API migration**: Replace fastn-core caching with fastn-cache API
- **Test suite implementation**: Comprehensive correctness verification
- **Performance benchmarking**: Automated measurement and regression detection

### 📋 Remaining Work
- **Complete fastn-core cleanup**: Remove old caching utilities
- **Advanced features**: Cache size limits, monitoring, compression
- **Documentation**: User guides and operational procedures

## Migration Strategy (Updated)

### Phase 1: Foundation ✅ COMPLETE
- ✅ Create fastn-cache crate with comprehensive design
- ✅ Implement storage and dependency tracking modules  
- ✅ Add fastn-cache dependency to fastn-core
- ✅ Enable optional caching with --enable-cache flag

### Phase 2: Testing & Validation 🚧 IN PROGRESS
- 🚧 Implement comprehensive test suite (10 critical scenarios)
- 🚧 Verify cache correctness under all conditions
- 🚧 Performance benchmarking and regression testing
- 🚧 Real-world validation with fastn.com

### Phase 3: Full Migration (Future)
- Replace all fastn-core caching with fastn-cache API
- Remove old caching utilities from fastn-core
- Enable caching by default when proven safe

### Phase 4: Advanced Features (Future)
- Cache size management and cleanup
- Performance monitoring and metrics
- Distributed cache for CI/CD systems

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

## Production Safety & Testing Strategy

### Critical Risk Assessment
**fastn 0.4 is used in production environments. Cache-related bugs are hard to debug and can cause:**
- Wrong content served (cache pollution between projects)
- Stale content after file changes (dependency tracking failures)  
- Build failures (incremental build regressions)
- Silent performance degradation

### Test Plan for Production Confidence

#### Phase 1: Cache Correctness Tests (CRITICAL)

**Test 1: Basic Cache Invalidation**
```bash
# Scenario: File change invalidates cache
1. Create test project: index.ftd imports hero.ftd  
2. Build with --enable-cache (measure performance)
3. Modify hero.ftd content
4. Request index.ftd 
5. VERIFY: Returns updated content (not stale cache)
6. VERIFY: Performance still good after invalidation
```

**Test 2: Dependency Chain Invalidation**
```bash
# Scenario: Deep dependency change propagates correctly
1. Create chain: index.ftd → hero.ftd → common.ftd → base.ftd
2. Cache all files (verify cache hits)
3. Modify base.ftd (root dependency)
4. Request index.ftd
5. VERIFY: Entire chain recompiled correctly
6. VERIFY: No files missed in invalidation
```

**Test 3: Multi-Project Cache Isolation**
```bash
# Scenario: Projects with same package name don't interfere  
1. Create project A: package "hello-world", content "A"
2. Create project B: package "hello-world", content "B"
3. Build both with caching
4. Modify project A files
5. Request project B content
6. VERIFY: Project B unaffected by A's changes
7. VERIFY: Project B serves correct content
```

**Test 4: Package Update Resilience**
```bash
# Scenario: fastn update invalidates affected caches
1. Create project with external dependencies
2. Cache all content
3. Simulate package update (touch .packages/*/files)
4. Request cached content
5. VERIFY: Cache invalidated and content recompiled
6. VERIFY: New package changes reflected
```

**Test 5: Configuration Change Detection**
```bash
# Scenario: FASTN.ftd changes invalidate cache appropriately
1. Cache project content
2. Modify FASTN.ftd (change imports, settings)
3. Request content
4. VERIFY: Cache invalidated due to config change
5. VERIFY: New configuration applied correctly
```

#### Phase 2: Build System Tests

**Test 6: Incremental Build Correctness**
```bash
# Scenario: Only affected files rebuilt
1. Create project with 10+ interconnected FTD files
2. Run fastn build (record all files built)
3. Modify one file
4. Run fastn build again
5. VERIFY: Only modified file + dependents rebuilt
6. VERIFY: Build output identical to full rebuild
```

**Test 7: Build Cache Persistence**
```bash
# Scenario: Build cache survives restarts
1. Run fastn build (populate cache)
2. Restart/simulate clean environment
3. Run fastn build again  
4. VERIFY: Cache used appropriately
5. VERIFY: Build time significantly reduced
```

#### Phase 3: Stress & Edge Case Tests

**Test 8: Concurrent Access**
```bash
# Scenario: Multiple fastn instances don't corrupt cache
1. Start multiple fastn serve instances
2. Concurrent requests to same files
3. VERIFY: No cache corruption
4. VERIFY: All responses correct
```

**Test 9: Cache Directory Behavior**
```bash
# Scenario: Verify cache directory naming works correctly
1. Test in git repo → verify repo-based naming
2. Test outside git → verify fallback naming
3. Test subdirectory projects → verify relative paths
4. VERIFY: Each scenario gets correct, isolated cache
```

**Test 10: Error Recovery**
```bash  
# Scenario: Recovery from cache corruption
1. Create valid cache
2. Corrupt cache files (invalid JSON, truncated files)
3. Request content
4. VERIFY: Graceful fallback to compilation
5. VERIFY: New valid cache created
```

### Testing Implementation Strategy

#### Option A: Shell-Based Test Suite (Recommended)
```bash
tests/
├── cache-correctness/
│   ├── run-all-tests.sh
│   ├── test-basic-invalidation.sh
│   ├── test-dependency-chain.sh
│   ├── test-multi-project.sh
│   └── test-package-updates.sh
└── build-tests/
    ├── test-incremental-build.sh
    └── test-build-cache.sh
```

**Benefits:**
- Fast to implement and debug
- Tests real fastn binary behavior
- Easy to run locally and in CI
- Clear pass/fail results

#### Test Project Structure
```
test-fixtures/
├── basic-project/         # Simple index.ftd + hero.ftd
├── dependency-chain/      # Complex dependency tree
├── multi-package/         # Multiple test projects  
└── large-project/         # Performance testing
```

### Production Safety Measures

#### Default Behavior: SAFE
- **Caching disabled by default** (--enable-cache opt-in)
- **Incremental build enabled** (low risk, high benefit)
- **Cache isolation ensures** no cross-project issues

#### Rollback Strategy
- **Feature flag**: Can disable caching via environment variable
- **Cache clearing**: fastn clean command for troubleshooting  
- **Monitoring**: Performance and correctness metrics

#### Staged Rollout Plan
1. **Internal testing**: Comprehensive test suite
2. **Beta users**: Optional caching with monitoring
3. **Gradual enable**: Once confidence established
4. **Full deployment**: Default caching when proven safe

### Success Criteria for Production Release

#### Functional Correctness
- [ ] All 10 test scenarios pass consistently
- [ ] No stale content served in any test case
- [ ] Cache invalidation works for all dependency types
- [ ] Multi-project isolation verified

#### Performance Verification  
- [ ] 100x+ performance improvement maintained
- [ ] No performance regression in edge cases
- [ ] Incremental build reduces build time by >90%

#### Production Readiness
- [ ] fastn.com builds and serves correctly with caching
- [ ] No regressions in existing fastn 0.4 functionality
- [ ] Clear error messages for cache issues
- [ ] Documentation updated for operations teams

---

**Only when ALL tests pass should we consider this ready for production fastn 0.4 users.**