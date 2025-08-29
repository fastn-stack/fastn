//! # Account HTTP Routes
//!
//! HTTP handlers for account web interface.

impl crate::Account {
    /// Route HTTP requests for this account
    ///
    /// # Parameters
    /// - `request`: The HTTP request to handle
    /// - `requester`: Optional PublicKey of who made the request
    ///   - `None`: Local access (full permissions)
    ///   - `Some(key)`: Remote P2P access (limited permissions based on key)
    pub async fn route_http(
        &self,
        request: &fastn_router::HttpRequest,
        requester: Option<&fastn_id52::PublicKey>,
    ) -> Result<fastn_router::HttpResponse, crate::AccountHttpError> {
        let primary_id52 = self.primary_id52().await.unwrap_or_default();

        let body = format!(
            "📧 Account Web Interface\n\n\
            Account ID: {}\n\
            Path: {}\n\
            Method: {}\n\
            Host: {}\n\
            Access Level: {}\n\
            Requester: {}\n\
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
            primary_id52, request.path, request.method, request.host, access_level, requester_info
        );

        Ok(fastn_router::HttpResponse::ok(body))
    }
}
