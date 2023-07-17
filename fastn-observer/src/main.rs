fn main() {
    fastn_observer::observe();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(outer_main(2, 3))
}

#[tracing::instrument]
async fn outer_main(time1: u64, time2: u64) {
    tracing::info!(time1, time2);
    test(time1, time2).await;
    tracing::info!("we are done");
}

#[tracing::instrument]
async fn foo(time: u64) {
    tracing::info!(time);
    tokio::time::sleep(std::time::Duration::from_secs(time)).await;
    tracing::info!(tag = "we are done");
}

#[tracing::instrument]
async fn test(time1: u64, time2: u64) {
    tracing::info!(hello = 10);
    tokio::time::sleep(std::time::Duration::from_secs(time1)).await;
    foo(time2).await;
    tracing::info!(awesome = "we are done");
}
