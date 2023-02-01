pub async fn update(config: &fastn::Config) -> fastn::Result<()> {
    if let Err(e) = std::fs::remove_dir_all(config.root.join(".packages")) {
        match e.kind() {
            std::io::ErrorKind::NotFound => {}
            _ => return Err(e.into()),
        }
    };

    let c = fastn::Config::read(None, false, None).await?;
    if c.package.dependencies.is_empty() {
        println!("No dependencies to update.")
    } else if c.package.dependencies.len() == 1 {
        println!("Updated the package dependency.")
    } else {
        println!("Updated {} dependencies.", c.package.dependencies.len())
    }

    Ok(())
}
