// Benchmark utilities for fastn JavaScript performance testing
// This file provides instrumentation and isolated testing capabilities

// Performance monitoring wrapper
const fastn_perf = {
    enabled:
        typeof window !== "undefined" &&
        (window.FASTN_BENCHMARK ||
            window.location.search.includes("benchmark=true")),
    timers: new Map(),
    counters: new Map(),

    mark(name) {
        if (this.enabled && typeof performance !== "undefined") {
            performance.mark(`fastn-${name}-start`);
        }
    },

    measure(name) {
        if (this.enabled && typeof performance !== "undefined") {
            performance.mark(`fastn-${name}-end`);
            try {
                performance.measure(
                    `fastn-${name}`,
                    `fastn-${name}-start`,
                    `fastn-${name}-end`,
                );
            } catch (e) {
                // Ignore timing errors in benchmarks
            }
        }
    },

    count(name) {
        if (this.enabled) {
            this.counters.set(name, (this.counters.get(name) || 0) + 1);
        }
    },

    getCounter(name) {
        return this.counters.get(name) || 0;
    },

    clearCounters() {
        this.counters.clear();
    },

    getResults() {
        if (!this.enabled || typeof performance === "undefined") return {};

        const measures = performance
            .getEntriesByType("measure")
            .filter((entry) => entry.name.startsWith("fastn-"))
            .map((entry) => ({
                name: entry.name.replace("fastn-", ""),
                duration: entry.duration,
                startTime: entry.startTime,
            }));

        return {
            measurements: measures,
            counters: Object.fromEntries(this.counters),
        };
    },

    clear() {
        if (typeof performance !== "undefined") {
            performance.clearMarks();
            performance.clearMeasures();
        }
        this.counters.clear();
        this.timers.clear();
    },
};

const fastn_benchmark_api = {
    // Performance monitoring
    perf: fastn_perf,

    // Module isolation helpers
    modules: {
        reactive: null,
        dom: null,
        utils: null,
        events: null,
        virtual: null,
    },

    // Reactive system testing
    createClosure: function (func, execute = true) {
        if (typeof fastn !== "undefined" && fastn.closure) {
            return new fastn.closure(func, execute);
        }
        throw new Error("fastn.closure not available");
    },

    createMutable: function (val) {
        if (typeof fastn !== "undefined" && fastn.mutable) {
            return new fastn.mutable(val);
        }
        throw new Error("fastn.mutable not available");
    },

    createMutableList: function (list = []) {
        if (typeof fastn !== "undefined" && fastn.mutableList) {
            return new fastn.mutableList(list);
        }
        throw new Error("fastn.mutableList not available");
    },

    // DOM operations testing
    createNode: function (kind) {
        if (typeof fastn_utils !== "undefined") {
            return fastn_utils.htmlNode(kind);
        }
        throw new Error("fastn_utils not available");
    },

    createStyle: function (cssClass, obj) {
        if (typeof fastn_utils !== "undefined") {
            return fastn_utils.createStyle(cssClass, obj);
        }
        throw new Error("fastn_utils.createStyle not available");
    },

    // Event handling testing
    simulateResize: function () {
        if (typeof window !== "undefined" && window.onresize) {
            return window.onresize();
        }
        return null;
    },

    triggerClosureUpdate: function (closure) {
        if (closure && typeof closure.update === "function") {
            return closure.update();
        }
        throw new Error("Invalid closure object");
    },

    // Utility testing
    getStaticValue: function (obj) {
        if (typeof fastn_utils !== "undefined") {
            return fastn_utils.getStaticValue(obj);
        }
        throw new Error("fastn_utils not available");
    },

    // CSS system testing
    getCSSCacheSize: function () {
        if (typeof fastn_css !== "undefined") {
            return fastn_css.getCacheSize();
        }
        return 0;
    },

    clearCSSCache: function () {
        if (typeof fastn_css !== "undefined") {
            fastn_css.clearCache();
        }
    },

    // Event system testing
    clearEventHandlers: function (type) {
        if (typeof fastn_events !== "undefined") {
            fastn_events.clear(type);
        }
    },

    getEventHandlerCount: function (type) {
        if (typeof fastn_events !== "undefined") {
            return fastn_events.handlers[type]
                ? fastn_events.handlers[type].length
                : 0;
        }
        return 0;
    },

    // Performance helpers
    clearPerformanceMarks: function () {
        fastn_perf.clear();
    },

    getPerformanceEntries: function () {
        return fastn_perf.getResults();
    },

    // Test data generators
    generateTestData: {
        largeArray: function (size = 1000) {
            return Array.from({ length: size }, (_, i) => ({
                id: i,
                value: `item-${i}`,
                data: Math.random(),
            }));
        },

        deepObject: function (depth = 5) {
            let obj = { value: "leaf" };
            for (let i = 0; i < depth; i++) {
                obj = { nested: obj, level: i };
            }
            return obj;
        },

        complexCSSProps: function () {
            return {
                "background-color": "#ff0000",
                "border-radius": "5px",
                "box-shadow": "0 2px 4px rgba(0,0,0,0.1)",
                margin: "10px",
                padding: "20px",
                "font-size": "14px",
                "line-height": "1.5",
            };
        },
    },
};

// Enable benchmark mode detection
if (typeof window !== "undefined") {
    window.FASTN_BENCHMARK =
        window.FASTN_BENCHMARK ||
        window.location.search.includes("benchmark=true") ||
        window.location.hostname === "localhost";

    window.fastn_benchmark_api = fastn_benchmark_api;
}
