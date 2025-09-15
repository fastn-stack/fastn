#[fastn_p2p::main(
    logging = "debug",
    shutdown_timeout = "60s"
)]
async fn main() -> eyre::Result<()> {
    println!("Hello from configured fastn_p2p::main macro!");
    println!("This should have debug logging enabled and 60s shutdown timeout");
    
    // Test configuration is working
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    println!("Configuration test completed successfully");
    Ok(())
}