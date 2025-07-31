# Fastn JavaScript Benchmarking

This document explains how to use the new benchmarking infrastructure added to
fastn's JavaScript runtime.

## Overview

The benchmarking system provides:
- **Performance instrumentation** for core operations
- **Memory profiling** capabilities
- **CSS caching** with hit/miss tracking
- **Event system** performance monitoring

## Quick Start

### 1. Enable Benchmark Mode

Add `?benchmark=true` to your URL or set `window.FASTN_BENCHMARK = true` in console.

### 2. Run Individual Tests

```javascript
// Create reactive objects
const mut = fastn_benchmark_api.createMutable(42);
const closure = fastn_benchmark_api.createClosure(() => mut.get() * 2);

// Run CSS tests
const css = fastn_benchmark_api.createStyle('test-class', {
    color: 'red',
    'font-size': '14px'
});

// Check performance results
console.log(fastn_benchmark_api.getPerformanceEntries());
```

### 3. Monitor Performance

```javascript
// Clear previous measurements
fastn_benchmark_api.clearPerformanceMarks();

// Perform operations to test
for (let i = 0; i < 1000; i++) {
    const mut = fastn_benchmark_api.createMutable(i);
    mut.set(i * 2);
}

// View results
const results = fastn_benchmark_api.getPerformanceEntries();
console.log('Timing measurements:', results.measurements);
console.log('Operation counters:', results.counters);
```

## Performance Monitoring

The system automatically tracks performance for these core operations:

### Timing Metrics
- `closure-update` - Time spent updating closures
- `mutable-set` - Time spent setting mutable values  
- `css-creation` - Time spent generating CSS
- `resize-handler` - Time spent processing resize events

### Counters
- `closure-updates` - Number of closure updates
- `mutable-created` - Number of mutable variable created
- `mutable-sets` - Number of mutable variable updates
- `css-creations` - Number of CSS classes generated
- `css-cache-hits/misses` - CSS cache effectiveness

## API Reference

### `fastn_benchmark_api`

Core benchmarking functions:

```javascript
// Reactive system
createClosure(func, execute) // Create new closure
createMutable(val)          // Create new mutable variable
createMutableList(list)     // Create new mutable list

// DOM operations  
createNode(kind)            // Create DOM node
createStyle(cssClass, obj)  // Generate CSS with caching

// Event testing
simulateResize()            // Trigger resize handler
clearEventHandlers(type)    // Clear event handlers

// Utilities
getStaticValue(obj)         // Convert to static value
getMemoryUsage()           // Get current memory stats
clearPerformanceMarks()    // Reset performance data

// Test data generators
generateTestData.largeArray(size)     // Large array for testing
generateTestData.deepObject(depth)    // Nested object for testing  
generateTestData.complexCSSProps()    // Complex CSS properties
```

### `fastn_perf`

Performance monitoring system:

```javascript
// Manual timing
fastn_perf.mark('my-operation');
// ... perform operation ...
fastn_perf.measure('my-operation');

// Counters
fastn_perf.count('my-counter');
console.log(fastn_perf.getCounter('my-counter'));

// Get all results
const results = fastn_perf.getResults();
console.log(results.measurements, results.counters);

// Clear data
fastn_perf.clear();
```

### `fastn_css`

CSS system with caching:

```javascript
// Usage (automatically used by fastn_utils.createStyle)
fastn_css.createStyle(className, properties)
fastn_css.clearCache()              // Clear CSS cache
fastn_css.getCacheSize()            // Get cache size
fastn_css.getStats()                // Get hit/miss stats
fastn_css.getCacheHitRatio()        // Get cache efficiency
```

### `fastn_events` 

Event system for testing:

```javascript
// Register/trigger events
fastn_events.register(type, handler, metadata)
fastn_events.trigger(type, event, targetHandlers)

// Testing utilities
fastn_events.clear(type)            // Clear handlers
fastn_events.getHandlerCount(type)  // Get handler count
fastn_events.getStats()             // Get event statistics
```

## Best Practices

### 1. Baseline Measurements
Always establish baseline performance before making changes:

```javascript
// Clear previous measurements
fastn_benchmark_api.clearPerformanceMarks();

// Record baseline
const baselineCounters = fastn_perf.getCounter('mutable-sets');
// Make changes...
const updatedCounters = fastn_perf.getCounter('mutable-sets');
console.log('Change in operations:', updatedCounters - baselineCounters);
```

### 2. Statistical Significance  
Run tests multiple times to get stable measurements:

```javascript
const measurements = [];
for (let i = 0; i < 5; i++) {
    fastn_benchmark_api.clearPerformanceMarks();
    // Perform test operations...
    const results = fastn_benchmark_api.getPerformanceEntries();
    measurements.push(results);
}
// Analyze variance in measurements
```

### 3. Realistic Test Data
Use representative data sizes and structures:

```javascript
// Test with realistic list sizes
const largeList = fastn_benchmark_api.generateTestData.largeArray(10000);
const mut = fastn_benchmark_api.createMutableList(largeList);
```

## Troubleshooting

### Benchmarks Not Running
- Check `window.FASTN_BENCHMARK` is true
- Verify performance API is available (`typeof performance !== 'undefined'`)
- Check browser console for errors

### Inaccurate Results  
- Disable browser extensions that might interfere
- Close other tabs to reduce resource contention
- Run tests in private/incognito mode
- Use dedicated benchmark browser profile

### Memory Issues
- Clear caches between test runs
- Force garbage collection if available (`gc()` in dev tools)
- Monitor for memory leaks in long-running tests

## Contributing

When adding performance instrumentation to new code:

1. Add `fastn_perf.mark()` and `fastn_perf.measure()` calls around critical operations
2. Use `fastn_perf.count()` to track operation frequency  
3. Add new functions to `fastn_benchmark_api` for isolated testing
4. Update this documentation

The benchmarking system provides performance insights to help optimize fastn's JavaScript runtime effectively.
