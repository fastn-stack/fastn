//! # Account HTTP Routes
//!
//! HTTP handlers for account web interface.

impl crate::Account {
    /// Route HTTP requests for this account
    pub async fn route_http(
        &self,
        request: &fastn_router::HttpRequest,
    ) -> Result<fastn_router::HttpResponse, crate::AccountHttpError> {
        let primary_id52 = self.primary_id52().await.unwrap_or_default();

        let body = format!(
            "📧 Account Web Interface\n\n\
            Account ID: {}\n\
            Path: {}\n\
            Method: {}\n\
            Host: {}\n\
            Type: Account\n\n\
            This is a fastn account web interface.\n\
            Email management features will be implemented here.\n\n\
            Available features:\n\
            - Email inbox and folders (coming soon)\n\
            - Compose and send emails (coming soon)\n\
            - Account settings (coming soon)\n\
            - Alias management (coming soon)\n\n\
            Current capabilities:\n\
            - P2P email delivery ✅\n\
            - SMTP email processing ✅\n\
            - Email storage and indexing ✅",
            primary_id52, request.path, request.method, request.host
        );

        Ok(fastn_router::HttpResponse::ok(body))
    }
}
