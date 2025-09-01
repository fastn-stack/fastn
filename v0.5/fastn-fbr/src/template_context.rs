//! # Template Context Trait
//!
//! Generic trait for providing context data to templates.

/// Generic context trait for template rendering
/// 
/// This allows any object (Account, Rig, etc.) to provide template context data
/// without fastn-fbr depending on specific crate types.
#[async_trait::async_trait] 
pub trait TemplateContext: Send + Sync {
    /// Convert object to template context data
    async fn to_template_context(&self) -> serde_json::Value;
}