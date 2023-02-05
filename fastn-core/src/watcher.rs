pub type WatcherSender = (usize, tokio::sync::mpsc::Sender<()>);
static WATCHER: once_cell::sync::Lazy<(
    tokio::sync::mpsc::Sender<WatcherSender>,
    tokio::sync::mpsc::Sender<usize>,
)> = once_cell::sync::Lazy::new(watcher);
const POLL_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(30 * 1000); // 30 seconds
static GLOBAL_POLL_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn watcher() -> (
    tokio::sync::mpsc::Sender<WatcherSender>,
    tokio::sync::mpsc::Sender<usize>,
) {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<WatcherSender>(32);
    let (g_tx, mut g_rx) = tokio::sync::mpsc::channel::<usize>(32);
    let (f_tx, mut f_rx) = tokio::sync::mpsc::channel::<()>(32);

    if fastn_core::utils::is_test() {
        // we do not want to run the watcher in tests
        return (tx, g_tx);
    }

    tokio::spawn(async move {
        let _watcher = create_watcher(f_tx); // watcher only works as long as it is not dropped
        let mut polls: std::collections::HashMap<usize, tokio::sync::mpsc::Sender<()>> =
            Default::default();

        loop {
            tokio::select! {
                Some((id, v)) = rx.recv() => {
                    // a new http poll request has joined, lets add them to pending watchers
                    polls.insert(id, v);
                    println!("enqueued new poll request");
                }
                Some(id) = g_rx.recv() => {
                    // a poll request has been completed, lets remove it from pending watchers
                    polls.remove(&id);
                    println!("removed poll request");
                }
                Some(()) = f_rx.recv() => {
                    // some file event has happened, lets inform all pending watchers
                    println!("file event, informing {} pending polls", polls.len());
                    for p in polls.values() {
                        if let Err(e) = p.send(()).await {
                            eprintln!("watcher: failed to send signal: {}", e);
                        }
                    }
                    polls.clear();
                }
                else => {
                    println!("watcher: exiting");
                    break;
                }
            }
        }
    });

    (tx, g_tx)
}

fn create_watcher(f_tx: tokio::sync::mpsc::Sender<()>) -> notify::RecommendedWatcher {
    use notify::Watcher;

    let mut watcher = notify::recommended_watcher(move |_res| {
        if let Err(e) = f_tx.blocking_send(()) {
            eprintln!("watcher: failed to send signal: {}", e);
        }
    })
    .expect("watcher: failed to create watcher");

    watcher
        .watch(
            &std::path::PathBuf::from(""), // TODO: how to get the root path?
            notify::RecursiveMode::Recursive,
        )
        .expect("watcher: failed to watch");

    watcher
}

fn next_id() -> usize {
    GLOBAL_POLL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

pub async fn poll() -> fastn_core::Result<fastn_core::http::Response> {
    let id = next_id();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(32);

    WATCHER.0.send((id, tx)).await?;

    let got_something = tokio::time::timeout(POLL_TIMEOUT, rx.recv()).await.is_ok();
    if !got_something {
        WATCHER.1.send(id).await?;
    }

    fastn_core::http::api_ok(got_something)
}
