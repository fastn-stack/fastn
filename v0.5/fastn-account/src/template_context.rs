//! # Account Template Context Implementation

impl crate::Account {
    /// Create template context with account data
    pub async fn create_template_context(&self) -> fastn_fbr::TemplateContext {
        let primary_id52 = self.primary_id52().await.unwrap_or_default();
        let aliases = self.aliases().await;

        // Create rich context data for templates
        let account_data = serde_json::json!({
            "account": {
                "primary_id52": primary_id52,
                "aliases": aliases.iter().map(|a| serde_json::json!({
                    "id52": a.id52(),
                    "name": a.name(),
                    "reason": a.reason(),
                    "is_primary": a.is_primary(),
                })).collect::<Vec<_>>(),
                "path": self.path().await.display().to_string(),
            },
            "mail": {
                // TODO: Add email-related context data
                "folders": ["INBOX", "Sent", "Drafts", "Trash"],
                "unread_count": 0, // TODO: Get actual unread count
            },
            "p2p": {
                // TODO: Add P2P status context
                "status": "online",
                "connections": [], // TODO: Get active connections
            }
        });

        fastn_fbr::TemplateContext::new().insert("account", &account_data)
    }
}
