#[fastn_p2p::main]
async fn main() -> eyre::Result<()> {
    println!("Starting long-running service...");
    println!("Press Ctrl+C to test graceful shutdown");

    // Simulate a long-running service
    for i in 1..=20 {
        println!("Working... step {}/20", i);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    println!("Service completed normally");
    Ok(())
}
