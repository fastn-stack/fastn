mod client;
mod cmd;
mod proxy;
pub mod server;

pub use cmd::attach_cmd;
pub use proxy::proxy;

// on fastn-net we will have more than one protocol. fastn-proxy is to proxy is to connect to
// a remove fastn server running over fastn-net, and start an HTTP server that will proxy all the
// requests to the remote fastn server.
pub const FASTN_PROXY: &[u8] = b"fastn-proxy";
