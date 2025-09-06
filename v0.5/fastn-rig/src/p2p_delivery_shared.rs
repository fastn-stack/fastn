//! Shared P2P delivery functionality
//! 
//! Contains the proven reliable P2P delivery approach used by both CLI and poller

/// Deliver a single P2P message using fresh connection (proven reliable)
pub async fn deliver_p2p_message(
    sender_secret_key: fastn_id52::SecretKey,
    target_peer: &fastn_id52::PublicKey,
    message: fastn_account::AccountToAccountMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create fresh endpoint (no reuse complexity)
    let endpoint = fastn_net::get_endpoint(sender_secret_key).await?;
    
    // Create fresh coordination structures  
    let peer_stream_senders = std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
    let graceful = fastn_net::Graceful::new();
    
    // Establish P2P stream using proven approach
    let (mut send, mut recv) = fastn_net::get_stream(
        endpoint,
        fastn_net::Protocol::AccountToAccount.into(),
        target_peer.id52(),
        peer_stream_senders,
        graceful,
    ).await?;
    
    // Send message
    let message_json = serde_json::to_string(&message)?;
    send.write_all(message_json.as_bytes()).await?;
    send.write_all(b"\n").await?;
    
    // Get response
    let response = fastn_net::next_string(&mut recv).await?;
    
    // Validate response
    let response_data: fastn_account::EmailDeliveryResponse = serde_json::from_str(&response)?;
    
    match response_data.status {
        fastn_account::DeliveryStatus::Accepted => Ok(()),
        fastn_account::DeliveryStatus::Rejected { reason } => {
            Err(format!("P2P delivery rejected: {}", reason).into())
        }
    }
}