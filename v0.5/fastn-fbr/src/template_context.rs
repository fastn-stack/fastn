//! # Template Context
//!
//! Generic context object for template rendering.

/// Template context data container
/// 
/// This allows passing arbitrary data to templates without fastn-fbr
/// depending on specific crate types (Account, Rig, etc.)
#[derive(Debug, Clone)]
pub struct TemplateContext {
    /// Context data as JSON value
    pub data: serde_json::Value,
}

impl TemplateContext {
    /// Create new empty template context
    pub fn new() -> Self {
        Self {
            data: serde_json::json!({}),
        }
    }
    
    /// Add data to template context
    pub fn insert<T: serde::Serialize>(mut self, key: &str, value: &T) -> Self {
        if let serde_json::Value::Object(ref mut map) = self.data {
            map.insert(key.to_string(), serde_json::to_value(value).unwrap_or(serde_json::Value::Null));
        }
        self
    }
    
    /// Convert to Tera context for rendering
    pub fn to_tera_context(&self) -> tera::Context {
        let mut context = tera::Context::new();
        
        if let serde_json::Value::Object(map) = &self.data {
            for (key, value) in map {
                context.insert(key, value);
            }
        }
        
        context
    }
}

impl Default for TemplateContext {
    fn default() -> Self {
        Self::new()
    }
}