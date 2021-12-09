pub async fn tracks(who: &str, whom: &str) -> fpm::Result<()> {
    let config = fpm::Config::read().await?;

    tokio::fs::create_dir_all(format!("{}/.tracks", config.root.as_str()).as_str()).await?;

    let snapshots = fpm::snaphot::get_latest_snapshots(config.root.as_str())?;
    check(config.root.as_str(), &snapshots, who, whom).await?;
    Ok(())
}

async fn check(
    base_path: &str,
    snapshots: &std::collections::BTreeMap<String, String>,
    who: &str,
    whom: &str,
) -> fpm::Result<()> {
    if !snapshots.contains_key(who) {
        eprintln!("Error: {} is not synced yet", who);
        println!("Suggestion: Run `fpm sync` to sync the files");
        return Ok(());
    }

    if !snapshots.contains_key(whom) {
        eprintln!("Error: {} is not synced yet", whom);
        println!("Suggestion: Run `fpm sync` to sync the files");
        return Ok(());
    }

    if who.contains('/') {
        let (dir, _) = who.rsplit_once('/').unwrap();
        std::fs::create_dir_all(format!("{}/.tracks/{}", base_path, dir))?;
    }

    let timestamp = snapshots.get(who).unwrap();

    let new_file_path = format!("{}/.tracks/{}", base_path, who.replace(".ftd", ".track"));

    write(whom, timestamp, &new_file_path).await?;
    println!("{} is now tracking {}", who, whom);

    Ok(())
}

async fn write(whom: &str, timestamp: &str, path: &str) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;
    let string = if tokio::fs::metadata(path).await.is_ok() {
        let existing_doc = tokio::fs::read_to_string(&path).await?;
        format!(
            "{}\n\n-- fpm.track: {}\nself-timestamp: {}",
            existing_doc, whom, timestamp
        )
    } else {
        format!(
            "-- import: fpm\n\n-- fpm.track: {}\nself-timestamp: {}",
            whom, timestamp
        )
    };

    let mut f = tokio::fs::File::create(path).await?;
    f.write_all(string.as_bytes()).await?;
    Ok(())
}
