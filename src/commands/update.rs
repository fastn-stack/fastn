pub async fn update(config: &fpm::Config) -> fpm::Result<()> {
    std::fs::remove_dir_all(config.root.join(".packages"))?;
    if fpm::Config::read().await.is_ok() {
        println!("package updated successfully");
    }
    Ok(())
}
