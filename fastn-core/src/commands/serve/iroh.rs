// on fastn-net we will have more than one protocol. fastn-proxy is to proxy is to connect to
// a remove fastn server running over fastn-net, and start an HTTP server that will proxy all the
// requests to the remote fastn server.
pub const FASTN_PROXY: &[u8] = b"fastn-proxy";

pub async fn init(_config: std::sync::Arc<fastn_core::Config>) -> fastn_core::Result<()> {
    // Get the key from `~/.fastn/keys/<pwd>/key`. Config should handle this.
    let ep = iroh::Endpoint::builder()
        .discovery_n0()
        .alpns(vec![FASTN_PROXY.to_vec()])
        .bind()
        .await?;

    let node_id = ep.node_id();

    if !fastn_core::utils::is_test() {
        println!("Listening on: fastn://{node_id}.");
        println!(
            "run `fastn proxy fastn://{node_id}` from any server in the world to connect to it.",
        );
        println!("use `fastn serve --no-fastn-net` to disable fastn-net.");
    }

    Ok(())
}
