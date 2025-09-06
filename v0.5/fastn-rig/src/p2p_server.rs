//! P2P server implementation using fastn-p2p

use crate::protocols::RigProtocol;
use crate::errors::P2PServerError;

/// Start P2P server for a single endpoint using fastn-p2p
/// 
/// This replaces the complex EndpointManager with clean fastn-p2p APIs
pub async fn start_p2p_listener(
    secret_key: fastn_id52::SecretKey,
    account_manager: std::sync::Arc<fastn_account::AccountManager>,
    message_tx: tokio::sync::mpsc::Sender<fastn_rig::P2PMessage>,
) -> Result<(), P2PServerError> {
    let public_key = secret_key.public_key();
    println!("🎧 Starting P2P listener for endpoint: {}", public_key.id52());

    // Listen on all rig protocols using the clean fastn-p2p API
    let protocols = vec![
        RigProtocol::EmailDelivery,
        RigProtocol::AccountMessage,
        RigProtocol::HttpProxy,
        RigProtocol::RigControl,
    ];

    let mut stream = fastn_p2p::listen!(secret_key, &protocols);

    println!("📡 P2P listener active, waiting for connections...");

    use futures_util::stream::StreamExt;
    while let Some(request_result) = stream.next().await {
        let request = request_result
            .map_err(|e| P2PServerError::RequestReceiveFailed { 
                message: e.to_string() 
            })?;

        println!("📨 Received {} request from {}", 
                request.protocol, 
                request.peer().id52());

        // Route based on protocol using clean matching
        match request.protocol {
            RigProtocol::EmailDelivery => {
                // Handle email delivery requests
                let account_manager_clone = account_manager.clone();
                let message_tx_clone = message_tx.clone();
                let peer_id = request.peer().clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_email_delivery_request(
                        request, 
                        account_manager_clone, 
                        message_tx_clone,
                        peer_id
                    ).await {
                        eprintln!("❌ Email delivery request failed: {}", e);
                    }
                });
            }
            RigProtocol::AccountMessage => {
                // Handle account messages
                println!("📩 Handling account message (TODO: implement)");
                // TODO: Implement account message handling
            }
            RigProtocol::HttpProxy => {
                // Handle HTTP proxy requests  
                println!("🌐 Handling HTTP proxy (TODO: implement)");
                // TODO: Implement HTTP proxy handling
            }
            RigProtocol::RigControl => {
                // Handle rig control messages
                println!("🎛️ Handling rig control (TODO: implement)");
                // TODO: Implement rig control handling
            }
        }
    }

    println!("🔚 P2P listener shutting down");
    Ok(())
}

/// Handle email delivery requests using the existing account message infrastructure
async fn handle_email_delivery_request(
    request: fastn_p2p::Request<RigProtocol>,
    _account_manager: std::sync::Arc<fastn_account::AccountManager>,
    message_tx: tokio::sync::mpsc::Sender<fastn_rig::P2PMessage>,
    peer_id: fastn_id52::PublicKey,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Use the existing message processing infrastructure
    request.handle(|msg: fastn_account::AccountToAccountMessage| async move {
        println!("📧 Processing email delivery message");
        
        // Convert to P2PMessage for existing infrastructure
        let p2p_msg = fastn_rig::P2PMessage {
            peer_id52: peer_id,
            our_endpoint: fastn_id52::SecretKey::generate().public_key(), // TODO: Get correct endpoint
            owner_type: fastn_rig::OwnerType::Account, // TODO: Get correct owner type
            message: serde_json::to_vec(&msg)
                .map_err(|e| crate::email_delivery_p2p::EmailDeliveryError {
                    message: format!("Serialization failed: {}", e),
                    code: "SERIALIZATION_ERROR".to_string(),
                })?,
        };

        // Send to existing message processing pipeline
        if let Err(_) = message_tx.send(p2p_msg).await {
            return Err(crate::email_delivery_p2p::EmailDeliveryError {
                message: "Message channel closed".to_string(),
                code: "CHANNEL_CLOSED".to_string(),
            });
        }

        // Return success response
        Ok(crate::email_delivery_p2p::EmailDeliveryResponse {
            email_id: "processed".to_string(), // TODO: Get actual email ID
            status: "accepted".to_string(),
        })
    }).await
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
}