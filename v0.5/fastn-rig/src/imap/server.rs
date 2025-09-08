//! IMAP server implementation

use std::sync::Arc;
use tokio::net::TcpListener;
use fastn_account::AccountManager;

/// Start IMAP server on specified port
pub async fn start_imap_server(
    account_manager: Arc<AccountManager>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📨 Starting IMAP server on port {}...", port);
    
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    println!("✅ IMAP server listening on 0.0.0.0:{}", port);
    
    loop {
        let (stream, addr) = listener.accept().await?;
        let account_manager = account_manager.clone();
        
        println!("🔗 New IMAP connection from {}", addr);
        
        tokio::spawn(async move {
            if let Err(e) = handle_imap_connection(stream, addr, account_manager).await {
                eprintln!("❌ IMAP connection error from {}: {}", addr, e);
            }
        });
    }
}

async fn handle_imap_connection(
    stream: tokio::net::TcpStream,
    client_addr: std::net::SocketAddr,
    account_manager: Arc<AccountManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::imap::session::ImapSession;
    
    let mut session = ImapSession::new(stream, client_addr, account_manager);
    session.handle().await
}