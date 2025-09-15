#[fastn_p2p::main]
async fn main() -> eyre::Result<()> {
    println!("Hello from fastn_p2p::main macro!");

    // Test basic functionality
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    println!("Main function completed successfully");
    Ok(())
}
