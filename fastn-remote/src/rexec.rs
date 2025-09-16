/// Execute command with separate stdout/stderr streams (automation mode)
pub async fn rexec(
    secret_key: fastn_id52::SecretKey,
    target: fastn_id52::PublicKey,
    command: &str,
) {
    todo!(
        "Execute command '{command}' in exec mode (separate streams) on {target} using {}",
        secret_key.id52()
    );
}
