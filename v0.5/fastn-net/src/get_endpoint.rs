/// Creates an Iroh endpoint configured for fastn networking.
///
/// This function creates an Iroh endpoint with:
/// - Local network discovery enabled
/// - N0 discovery (DHT-based) enabled  
/// - ALPN set to `/fastn/identity/0.1`
/// - The provided secret key for identity
///
/// # Errors
///
/// Returns an error if the endpoint fails to bind to the network.
pub async fn get_endpoint(secret_key: fastn_id52::SecretKey) -> eyre::Result<iroh::Endpoint> {
    // Convert fastn_id52::SecretKey to iroh::SecretKey
    let iroh_secret_key = iroh::SecretKey::from_bytes(&secret_key.to_bytes());

    match iroh::Endpoint::builder()
        .discovery_n0()
        .discovery_local_network()
        .alpns(vec![crate::APNS_IDENTITY.into()])
        .secret_key(iroh_secret_key)
        .bind()
        .await
    {
        Ok(ep) => Ok(ep),
        Err(e) => {
            // https://github.com/n0-computer/iroh/issues/2741
            // this is why you MUST NOT use anyhow::Error etc. in library code.
            Err(eyre::anyhow!("failed to bind to iroh network2: {e:?}"))
        }
    }
}
