/// Test the minimal fastn-context API needed for fastn-p2p integration
/// This validates our basic Context design before implementation

#[fastn_context::main]
async fn main() -> eyre::Result<()> {
    println!("Testing minimal fastn-context API...");
    
    // Global context should be automatically available
    let global_ctx = fastn_context::global();
    println!("Global context created: {}", global_ctx.name);
    
    // Test basic child creation with builder pattern
    let service_ctx = global_ctx.child("test-service");
    println!("Service context created: {}", service_ctx.name);
    
    // Test simple task spawning with shortcut
    service_ctx.spawn_child("simple-task", |task_ctx| async move {
        println!("Task 1 running with explicit context: {}", task_ctx.name);
        
        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        println!("Task 1 completed");
        
        // Context explicitly available for basic operations
        println!("Task context available: {}", task_ctx.name);
    });
    
    // Test builder pattern with explicit context passing
    service_ctx.child("test-session")
        .spawn(|task_ctx| async move {
            println!("Task 2 running with explicit context: {}", task_ctx.name);
            
            // Test cancellation handling with explicit context
            tokio::select! {
                _ = task_ctx.wait() => {
                    println!("Task 2 cancelled by explicit context");
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