use tracing::{debug, error, info, warn};
use tracing_forest::ForestLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};

#[tracing::instrument]
async fn recursive(param: i32) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()>>> {
    Box::pin(async move {
        if param < 0 {
            return;
        }
        // This recursion is not getting logged
        tracing::Span::current()
            .in_scope(|| async {
                recursive(param - 1).await;
            })
            .await;
    })
}

#[tracing::instrument]
async fn some_expensive_operation(id: u32) {
    tracing::Span::current().record("id", id);
    debug!("starting from `some_expensive_operation`");
    std::thread::sleep(std::time::Duration::from_secs(1));
    recursive(5).await;
    error!("exiting from `some_expensive_operation`");
}

#[tracing::instrument(fields(id))]
async fn conn(id: u32) {
    for i in 0..3 {
        info!(id, "running step {}", i);
        some_expensive_operation(id).await;
        info!(id, "ending step {}", i);
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    use tracing_tree::HierarchicalLayer;
    // tracing_subscriber::init(|builder| {
    //     builder
    //         .with_max_level(tracing::Level::DEBUG)
    //         // .with_env_filter("my_app_name=debug")
    //         .with_format(|buf, record, _| {
    //             writeln!(
    //                 buf,
    //                 "[{}][{}] {}",
    //                 record.level(),
    //                 record.target(),
    //                 record.args(),
    //             )
    //         })
    // });

    Registry::default()
        .with(HierarchicalLayer::default())
        .init();
    conn(5).await;
}
