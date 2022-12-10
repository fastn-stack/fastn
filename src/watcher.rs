static WATCHER: once_cell::sync::Lazy<tokio::sync::mpsc::Sender<tokio::sync::mpsc::Sender<()>>> =
    once_cell::sync::Lazy::new(watcher);

const POLL_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(30 * 1000); // 30 seconds

fn watcher() -> tokio::sync::mpsc::Sender<tokio::sync::mpsc::Sender<()>> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<tokio::sync::mpsc::Sender<()>>(32);

    tokio::spawn(async move {
        while let Some(v) = rx.recv().await {
            if let Err(e) = v.send(()).await {
                eprintln!("watcher error: {}", e);
                continue;
            };
            println!("got a message");
        }
        unreachable!("we should never come out of the while loop");
    });
    tx
}

pub async fn poll() -> fpm::Result<fpm::http::Response> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(32);

    WATCHER.send(tx).await?;

    fpm::http::api_ok(tokio::time::timeout(POLL_TIMEOUT, rx.recv()).await.is_ok())
}
