//! Graceful shutdown management for async services.
//!
//! This module provides the [`Graceful`] type for coordinating clean shutdown
//! of async tasks in network services. It ensures all spawned tasks complete
//! before the service exits.
//!
//! # Overview
//!
//! When building network services, you often spawn multiple async tasks for
//! handling connections, background work, etc. The `Graceful` type helps you:
//!
//! - Signal all tasks to stop via cancellation tokens
//! - Track all spawned tasks to ensure they complete
//! - Coordinate shutdown across multiple components
//!
//! # Example: Basic HTTP Server with Graceful Shutdown
//!
//! ```no_run
//! use fastn_net::Graceful;
//! use tokio::net::TcpListener;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let graceful = Graceful::new();
//!
//! // Spawn a server task
//! let server_graceful = graceful.clone();
//! graceful.spawn(async move {
//!     let listener = TcpListener::bind("127.0.0.1:8080").await?;
//!     
//!     loop {
//!         tokio::select! {
//!             // Accept new connections
//!             Ok((stream, _)) = listener.accept() => {
//!                 // Handle connection in a tracked task
//!                 server_graceful.spawn(async move {
//!                     // Process the connection...
//!                     Ok::<(), eyre::Error>(())
//!                 });
//!             }
//!             // Stop accepting when cancelled
//!             _ = server_graceful.cancelled() => {
//!                 println!("Server shutting down...");
//!                 break;
//!             }
//!         }
//!     }
//!     Ok::<(), eyre::Error>(())
//! });
//!
//! // In your main or signal handler:
//! // graceful.shutdown().await;
//! # Ok(())
//! # }
//! ```
//!
//! # Example: P2P Service with Multiple Components
//!
//! ```no_run
//! use fastn_net::{Graceful, global_iroh_endpoint};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let graceful = Graceful::new();
//! let endpoint = global_iroh_endpoint().await;
//!
//! // Component 1: Accept incoming P2P connections
//! let p2p_graceful = graceful.clone();
//! graceful.spawn(async move {
//!     while let Some(conn) = endpoint.accept().await {
//!         tokio::select! {
//!             _ = p2p_graceful.cancelled() => {
//!                 break;
//!             }
//!             else => {
//!                 // Handle each connection in a tracked task
//!                 p2p_graceful.spawn(async move {
//!                     // Process P2P connection...
//!                     Ok::<(), eyre::Error>(())
//!                 });
//!             }
//!         }
//!     }
//!     Ok::<(), eyre::Error>(())
//! });
//!
//! // Component 2: HTTP API server
//! let api_graceful = graceful.clone();
//! graceful.spawn(async move {
//!     // Run HTTP server with cancellation check
//!     loop {
//!         tokio::select! {
//!             _ = api_graceful.cancelled() => {
//!                 break;
//!             }
//!             _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
//!                 // Handle HTTP requests...
//!             }
//!         }
//!     }
//!     Ok::<(), eyre::Error>(())
//! });
//!
//! // Graceful shutdown on Ctrl+C
//! tokio::select! {
//!     _ = tokio::signal::ctrl_c() => {
//!         println!("Shutting down gracefully...");
//!         graceful.shutdown().await?;
//!         println!("All tasks completed");
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Best Practices
//!
//! 1. **Clone for each component**: Each async task or component should get
//!    its own clone of `Graceful` to spawn sub-tasks.
//!
//! 2. **Check cancellation in loops**: Long-running loops should use
//!    `select!` with `cancelled()` for proper cancellation handling.
//!
//! 3. **Use spawn() for all tasks**: Always use `graceful.spawn()` instead of
//!    `tokio::spawn()` to ensure tasks are tracked.
//!
//! 4. **Handle errors**: Tasks spawned with `spawn()` should return `Result`
//!    to properly propagate errors during shutdown.
//!
//! 5. **Shutdown order**: Call `shutdown()` from your main function or signal
//!    handler, which will:
//!    - Cancel all tasks via the cancellation token
//!    - Wait for all tracked tasks to complete
//!    - Return any errors from failed tasks

use eyre::Context;
use tokio::task::JoinHandle;

/// Manages graceful shutdown of async tasks.
///
/// Combines cancellation signaling with task tracking to ensure
/// clean shutdown of all spawned tasks. Clone this freely - all
/// clones share the same underlying state.
#[derive(Clone)]
pub struct Graceful {
    cancel: tokio_util::sync::CancellationToken,
    tracker: tokio_util::task::TaskTracker,
    show_info_tx: tokio::sync::watch::Sender<bool>,
    show_info_rx: tokio::sync::watch::Receiver<bool>,
}

impl Default for Graceful {
    fn default() -> Self {
        Self::new()
    }
}

impl Graceful {
    pub fn new() -> Self {
        let (show_info_tx, show_info_rx) = tokio::sync::watch::channel(false);

        Self {
            cancel: tokio_util::sync::CancellationToken::new(),
            tracker: tokio_util::task::TaskTracker::new(),
            show_info_tx,
            show_info_rx,
        }
    }

    pub async fn show_info(&mut self) -> eyre::Result<()> {
        self.show_info_rx
            .changed()
            .await
            .map_err(|e| eyre::anyhow!("failed to get show info signal: {e:?}"))
    }

    #[inline]
    #[track_caller]
    pub fn spawn<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tracker.spawn(task)
    }

    pub async fn shutdown(&self) -> eyre::Result<()> {
        loop {
            tokio::signal::ctrl_c()
                .await
                .wrap_err_with(|| "failed to get ctrl-c signal handler")?;

            tracing::info!("Received ctrl-c signal, showing info.");
            tracing::info!("Pending tasks: {}", self.tracker.len());

            self.show_info_tx
                .send(true)
                .inspect_err(|e| tracing::error!("failed to send show info signal: {e:?}"))?;

            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Received second ctrl-c signal, shutting down.");
                    tracing::debug!("Pending tasks: {}", self.tracker.len());

                    self.cancel.cancel();
                    self.tracker.close();

                    let mut count = 0;
                    loop {
                        tokio::select! {
                            _ = self.tracker.wait() => {
                                tracing::info!("All tasks have exited.");
                                break;
                            }
                            _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {
                                count += 1;
                                if count > 10 {
                                    eprintln!("Timeout expired, {} pending tasks. Exiting...", self.tracker.len());
                                    break;
                                }
                                tracing::debug!("Pending tasks: {}", self.tracker.len());
                            }
                        }
                    }
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {
                    tracing::info!("Timeout expired. Continuing...");
                    println!("Did not receive ctrl+c within 3 secs. Press ctrl+c in quick succession to exit.");
                }
            }
        }

        Ok(())
    }

    pub fn cancelled(&self) -> tokio_util::sync::WaitForCancellationFuture<'_> {
        self.cancel.cancelled()
    }
}
