/// Test the minimal fastn-context API needed for fastn-p2p integration
/// This validates our basic Context design before implementation

#[fastn_context::main]
async fn main() -> eyre::Result<()> {
    println!("Testing minimal fastn-context API...");
    
    // Global context should be automatically available
    let global_ctx = fastn_context::global();
    println!("Global context created: {}", global_ctx.name);
    
    // Test basic child creation
    let service_ctx = global_ctx.child("test-service");
    println!("Service context created: {}", service_ctx.name);
    
    // Test task spawning with context inheritance
    service_ctx.spawn(async {
        println!("Task 1 running in service context");
        
        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        println!("Task 1 completed");
    });
    
    // Test nested contexts
    let session_ctx = service_ctx.child("test-session");
    session_ctx.spawn(async {
        println!("Task 2 running in session context");
        
        // Test cancellation handling  
        tokio::select! {
            _ = fastn_context::current().wait() => {
                println!("Task 2 cancelled by context");
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(200)) => {
                println!("Task 2 completed normally");
            }
        }
    });
    
    // Let tasks run briefly
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    // Test cancellation
    println!("Cancelling service context...");
    service_ctx.cancel();
    
    // Brief delay to see cancellation effects
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    println!("Minimal API test completed!");
    Ok(())
}