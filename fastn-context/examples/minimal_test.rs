/// Test the minimal fastn-context API needed for fastn-p2p integration
/// This validates our basic Context design before implementation

fn main() {
    println!("Testing minimal fastn-context API...");
    
    // Global context should be automatically available
    let global_ctx = fastn_context::global();
    println!("Global context created: {}", global_ctx.name);
    
    // Test basic child creation with builder
    global_ctx.child("test-service")
        .spawn(|service_ctx| async move {
            println!("Service context created: {}", service_ctx.name);
            
            // Test cancellation
            tokio::select! {
                _ = service_ctx.wait() => {
                    println!("Service context cancelled");
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                    println!("Service context completed");
                }
            }
        });
    
    // Test global context functionality
    println!("Global context is cancelled: {}", global_ctx.is_cancelled());
    
    println!("Basic API test completed!");
}