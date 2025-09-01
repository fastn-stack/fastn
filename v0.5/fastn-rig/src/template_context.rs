//! # Rig Template Context Implementation

impl crate::Rig {
    /// Create template context with rig data and functions
    pub async fn create_template_context(&self) -> fastn_fbr::TemplateContext {
        fastn_fbr::TemplateContext::new()
            // Add minimal static data
            .insert("request_time", &chrono::Utc::now().timestamp())
            
            // Register dynamic functions for rig data
            .register_function("rig_id52", |_args| {
                // TODO: Access actual rig data - for now return placeholder
                Ok(tera::Value::String("rig_id52_placeholder".to_string()))
            })
            .register_function("rig_owner", |_args| {
                // TODO: Access actual rig owner - return placeholder
                Ok(tera::Value::String("owner_id52_placeholder".to_string()))
            })
            .register_function("rig_accounts", |_args| {
                // TODO: Get actual account list - return placeholder
                Ok(tera::Value::Array(vec![
                    tera::Value::String("account1".to_string()),
                    tera::Value::String("account2".to_string()),
                ]))
            })
            .register_function("rig_endpoints", |_args| {
                // TODO: Get actual endpoint status - return placeholder
                Ok(tera::Value::Number(serde_json::Number::from(2)))
            })
    }
}