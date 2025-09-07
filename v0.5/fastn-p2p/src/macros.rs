/// Convenience macro for starting a P2P listener with automatic pinning
///
/// This macro combines `listen()` and `std::pin::pin!()` into a single call,
/// eliminating boilerplate for the common case.
///
/// # Example
///
/// ```rust,ignore
/// use futures_util::stream::StreamExt;
///
/// // Before: Two lines with mystifying pin! macro
/// let stream = fastn_p2p::listen(secret_key, &protocols)?;
/// let mut stream = std::pin::pin!(stream);
///
/// // After: One clean line
/// let mut stream = fastn_p2p::listen!(secret_key, &protocols)?;
///
/// while let Some(request) = stream.next().await {
///     // Handle requests...
/// }
/// ```
#[macro_export]
macro_rules! listen {
    ($secret_key:expr, $protocols:expr) => {{
        let stream = $crate::listen($secret_key, $protocols)?;
        std::pin::pin!(stream)
    }};
}
