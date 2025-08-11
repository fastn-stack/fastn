//! Test utilities for fastn-package
//!
//! This module contains helper functions and constructors that are only used in tests.

impl crate::Dependency {
    /// Creates a simple dependency for testing purposes
    pub fn new_for_test(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            capabilities: vec![],
            dependencies: vec![],
            auto_imports: vec![],
        }
    }
}

impl crate::Package {
    /// Creates a test package with the given name and dependencies
    pub fn new_for_test(name: impl Into<String>, dependencies: Vec<crate::Dependency>) -> Self {
        Self {
            name: name.into(),
            dependencies,
            auto_imports: vec![],
            favicon: None,
        }
    }
}
