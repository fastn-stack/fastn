//! # Template Context with Custom Functions
//!
//! Performance-oriented template context using Tera custom functions.

/// Function registry for template rendering
///
/// This allows registering functions that can be called from templates
/// for dynamic data fetching, avoiding pre-loading all data.
pub type TemplateFunctionRegistry = std::collections::HashMap<
    String,
    fn(&std::collections::HashMap<String, tera::Value>) -> tera::Result<tera::Value>,
>;

/// Template context with dynamic function support
#[derive(Default)]
pub struct TemplateContext {
    /// Custom functions available to templates
    pub functions: TemplateFunctionRegistry,
    /// Minimal static data (only request info, etc.)
    pub static_data: std::collections::HashMap<String, serde_json::Value>,
}

impl TemplateContext {
    /// Create new template context
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a custom function for templates
    pub fn register_function(
        mut self,
        name: &str,
        function: fn(&std::collections::HashMap<String, tera::Value>) -> tera::Result<tera::Value>,
    ) -> Self {
        self.functions.insert(name.to_string(), function);
        self
    }

    /// Add static data to context
    pub fn insert<T: serde::Serialize>(mut self, key: &str, value: &T) -> Self {
        self.static_data.insert(
            key.to_string(),
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }

    /// Convert to Tera context with registered functions
    pub fn to_tera_context(&self, template_engine: &mut tera::Tera) -> tera::Context {
        // Register all custom functions with the template engine
        for (name, function) in &self.functions {
            template_engine.register_function(name, *function);
        }

        // Create context with static data
        let mut context = tera::Context::new();
        for (key, value) in &self.static_data {
            context.insert(key, value);
        }

        context
    }
}
