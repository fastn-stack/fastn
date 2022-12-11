static WATCHER: once_cell::sync::Lazy<tokio::sync::mpsc::Sender<tokio::sync::mpsc::Sender<()>>> =
    once_cell::sync::Lazy::new(watcher);

const POLL_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(30 * 1000); // 30 seconds

fn watcher() -> tokio::sync::mpsc::Sender<tokio::sync::mpsc::Sender<()>> {
    use notify::Watcher;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<tokio::sync::mpsc::Sender<()>>(32);
    let (f_tx, mut f_rx) = tokio::sync::mpsc::channel::<()>(32);

    let mut watcher = notify::recommended_watcher(move |res| {
        // TODO: these are never getting fired, why?

        println!("watcher: {:?}", res);
        if let Err(e) = f_tx.blocking_send(()) {
            eprintln!("watcher: failed to send signal: {}", e);
        }
    })
    .expect("watcher: failed to create watcher");

    let to_watch = std::path::PathBuf::from("t/");
    watcher
        .watch(&to_watch, notify::RecursiveMode::Recursive)
        .expect("watcher: failed to watch");

    println!("watching: {:?}", to_watch.canonicalize());

    tokio::spawn(async move {
        let mut polls = vec![];

        loop {
            tokio::select! {
                Some(v) = rx.recv() => {
                    // a new http poll request has joined, lets add them to pending watchers
                    polls.push(v);
                    println!("enqueued new poll request");
                }
                Some(()) = f_rx.recv() => {
                    // some file event has happened, lets inform all pending watchers
                    println!("file event, informing {} pending polls", polls.len());
                    for p in polls {
                        if let Err(e) = p.send(()).await {
                            eprintln!("watcher: failed to send signal: {}", e);
                        }
                    }
                    polls = vec![];
                }
                else => {
                    println!("watcher: exiting");
                    break;
                }
            }
        }
    });
    tx
}

pub async fn poll() -> fpm::Result<fpm::http::Response> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(32);

    WATCHER.send(tx).await?;

    // TODO: when timing out send another signal to the watcher to make sure the handle is dropped
    //       or we can let the sender not fail when there is a dead handle, and also put a timeout
    //       on the receiver so every 30 seconds or so it cleans up all the dead handles.

    fpm::http::api_ok(tokio::time::timeout(POLL_TIMEOUT, rx.recv()).await.is_ok())
}
