async fn print_app_status() {
    println!("=== Application Status ===");
    println!("Uptime: 42 seconds");
    println!("Active tasks: 3");
    println!("Memory usage: 15.2 MB");
    println!("=========================");
}

#[fastn_p2p::main(
    logging = "info",
    shutdown_mode = "double_ctrl_c",
    status_fn = "print_app_status",
    double_ctrl_c_window = "3s"
)]
async fn main() -> eyre::Result<()> {
    println!("Starting service with double Ctrl+C mode...");
    println!("Press Ctrl+C once to see status, twice within 3s to shutdown");
    
    // Simulate a long-running service
    let mut counter = 0;
    loop {
        counter += 1;
        println!("Service running... iteration {}", counter);
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Use fastn_p2p::cancelled() to check for shutdown
        tokio::select! {
            _ = fastn_p2p::cancelled() => {
                println!("Service received shutdown signal, cleaning up...");
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(0)) => {
                // Continue loop
            }
        }
    }
    
    println!("Service completed gracefully");
    Ok(())
}