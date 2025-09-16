/// Interactive remote shell (PTY mode)
pub async fn rshell(
    secret_key: fastn_id52::SecretKey,
    target: fastn_id52::PublicKey,
    command: Option<&str>,
) {
    match command {
        Some(cmd) => todo!(
            "Execute command '{cmd}' in shell mode (PTY) on {target} using {}",
            secret_key.id52()
        ),
        None => todo!(
            "Start interactive shell session on {target} using {}",
            secret_key.id52()
        ),
    }
}
