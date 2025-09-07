//! P2P server implementation using fastn-p2p

use crate::errors::P2PServerError;
use crate::protocols::RigProtocol;

/// Start P2P server for a single endpoint using fastn-p2p
///
/// This replaces the complex EndpointManager with clean fastn-p2p APIs
pub async fn start_p2p_listener(
    secret_key: fastn_id52::SecretKey,
    account_manager: std::sync::Arc<fastn_account::AccountManager>,
) -> Result<(), P2PServerError> {
    let public_key = secret_key.public_key();
    println!(
        "🎧 Starting P2P listener for endpoint: {}",
        public_key.id52()
    );

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
        let request = request_result.map_err(|e| P2PServerError::RequestReceive {
            message: e.to_string(),
        })?;

        println!(
            "📨 Received {} request from {}",
            request.protocol,
            request.peer().id52()
        );

        // Route based on protocol using clean matching
        match request.protocol {
            RigProtocol::EmailDelivery => {
                // Handle email delivery directly using account manager
                let account_manager_clone = account_manager.clone();

                if let Err(e) = request
                    .handle(|msg| {
                        handle_email_message_direct(msg, account_manager_clone, public_key)
                    })
                    .await
                {
                    eprintln!("❌ Email delivery request failed: {e}");
                }
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

/// Handle email message directly using account manager (no more channel complexity)
async fn handle_email_message_direct(
    msg: fastn_account::AccountToAccountMessage,
    account_manager: std::sync::Arc<fastn_account::AccountManager>,
    our_endpoint: fastn_id52::PublicKey,
) -> Result<
    crate::email_delivery_p2p::EmailDeliveryResponse,
    crate::email_delivery_p2p::EmailDeliveryError,
> {
    println!("📧 Processing email delivery directly");

    // Generate a peer ID for now (TODO: get from actual request context)
    let peer_id = fastn_id52::SecretKey::generate().public_key();

    // Process email directly using account manager
    match account_manager
        .handle_account_message(&peer_id, &our_endpoint, msg)
        .await
    {
        Ok(_) => {
            println!("✅ Email processed successfully via fastn-p2p");
            Ok(crate::email_delivery_p2p::EmailDeliveryResponse {
                email_id: "processed".to_string(),
                status: "accepted".to_string(),
            })
        }
        Err(e) => {
            println!("❌ Email processing failed: {e}");
            Err(crate::email_delivery_p2p::EmailDeliveryError {
                message: format!("Processing failed: {e}"),
                code: "PROCESSING_ERROR".to_string(),
            })
        }
    }
}
