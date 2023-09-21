pub fn get_user_agent(product: &str) -> String {
    let cargo_version = env!("CARGO_PKG_VERSION");
    let target_os = std::env::consts::OS;
    let target_arch = std::env::consts::ARCH;
    format!(
        "vercel/remote {} {} {} {} ({})",
        cargo_version,
        product,
        env!("CARGO_PKG_VERSION"),
        target_os,
        target_arch
    )
}
