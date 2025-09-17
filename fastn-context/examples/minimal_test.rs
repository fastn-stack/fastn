/// Test the minimal fastn-context API needed for fastn-p2p integration
/// This validates our basic Context design before implementation

#[fastn_context::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing minimal fastn-context API...");

    // Global context should be automatically available
    let global_ctx = fastn_context::global();
    println!("Global context created: {}", global_ctx.name);

    // Test basic child creation with builder
    global_ctx
        .child("test-service")
        .spawn(|service_ctx| async move {
            println!("Service context created: {}", service_ctx.name);

            // Test cancellation with proper select pattern
            tokio::select! {
                _ = service_ctx.cancelled() => {
                    println!("Service context cancelled");
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                    println!("Service context completed");
                }
            }
        });

    // Test global context functionality
    println!("Global context is cancelled: {}", global_ctx.is_cancelled());

    // Give tasks time to run and build tree
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Test status display
    println!("\n=== Context Tree Status ===");
    let status = fastn_context::status();
    println!("{}", status);

    // Test persistence functionality
    global_ctx.spawn_child("persist-test", |task_ctx| async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        task_ctx.persist();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Test status with persisted contexts
    println!("\n=== Status with Persisted Contexts ===");
    let status_with_latest = fastn_context::status_with_latest();
    println!("{}", status_with_latest);

    println!("Basic API test completed!");
    Ok(())
}
