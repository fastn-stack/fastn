//! # Rig HTTP Routes
//!
//! HTTP handlers for rig management interface.

impl fastn_rig::Rig {
    /// Route HTTP requests for rig management
    ///
    /// # Parameters
    /// - `request`: The HTTP request to handle
    /// - `requester`: Optional PublicKey of who made the request
    ///   - `None`: Local access (full admin permissions)
    ///   - `Some(key)`: Remote P2P access (read-only public info)
    pub async fn route_http(
        &self,
        request: &fastn_router::HttpRequest,
        requester: Option<&fastn_id52::PublicKey>,
    ) -> Result<fastn_router::HttpResponse, crate::RigHttpError> {
        // Determine access level based on requester
        let access_level = match requester {
            None => "Local (Full Admin)",
            Some(key) => {
                if key.id52() == self.id52() || *key == *self.owner() {
                    "Authorized (Full Admin)"
                } else {
                    "Remote P2P (Public Info Only)"
                }
            }
        };
        
        let requester_info = match requester {
            None => "Local Browser".to_string(),
            Some(key) => key.id52(),
        };
        
        let body = format!(
            "⚙️ Rig Management Interface\n\n\
            Rig ID: {}\n\
            Owner: {}\n\
            Path: {}\n\
            Method: {}\n\
            Host: {}\n\
            Access Level: {}\n\
            Requester: {}\n\
            Type: Rig\n\n\
            This is the fastn rig management interface.\n\
            System administration features will be implemented here.\n\n\
            Available features:\n\
            - Account management (coming soon)\n\
            - P2P network status (coming soon)\n\
            - Email delivery monitoring (coming soon)\n\
            - System configuration (coming soon)\n\n\
            Current capabilities:\n\
            - P2P email delivery poller ✅\n\
            - Multi-account management ✅\n\
            - Endpoint lifecycle management ✅\n\
            - Real-time email processing ✅",
            self.id52(),
            self.owner(),
            request.path,
            request.method,
            request.host
        );

        Ok(fastn_router::HttpResponse::ok(body))
    }
}
