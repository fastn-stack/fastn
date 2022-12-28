use tracing::{debug, error, info, warn};
use tracing_forest::ForestLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};

#[tracing::instrument]
fn recursive(param: i32) {
    info!(msg = "inside recursive", param = param);
    std::thread::sleep(std::time::Duration::from_secs(1));
    if param < 0 {
        return;
    }
    // This span is not recording the param is not getting logged
    recursive(param - 1);
}

#[tracing::instrument]
async fn some_expensive_operation(id: u32) {
    tracing::Span::current().record("id", id);
    debug!("starting from `some_expensive_operation`");
    recursive(5);
    error!("exiting from `some_expensive_operation`");
}

#[tracing::instrument(fields(id))]
async fn conn(id: u32) {
    for i in 0..2 {
        info!(id, "running step {}", i);
        some_expensive_operation(id).await;
        info!(id, "ending step {}", i);
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
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

    Registry::default().with(ForestLayer::default()).init();
    conn(5).await;
}
