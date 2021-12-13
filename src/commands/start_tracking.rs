pub async fn start_tracking(config: &fpm::Config, who: &str, whom: &str) -> fpm::Result<()> {
    tokio::fs::create_dir_all(config.track_dir()).await?;

    let snapshots = fpm::snapshot::get_latest_snapshots(config).await?;
    check(config.root.as_str(), &snapshots, who, whom).await?;
    Ok(())
}

async fn check(
    base_path: &str,
    snapshots: &std::collections::BTreeMap<String, u128>,
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

    let who_ext = who.rsplit_once('.').unwrap_or(("", "")).1;
    let whom_ext = whom.rsplit_once('.').unwrap_or(("", "")).1;

    if !who_ext.eq(whom_ext) {
        eprintln!(
            "Error: file extension doesn't match. Expected target of extension `.{}` found `.{}`",
            who_ext, whom_ext
        );
        return Ok(());
    }

    if who.contains('/') {
        let (dir, _) = who.rsplit_once('/').unwrap();
        std::fs::create_dir_all(
            camino::Utf8PathBuf::from(base_path)
                .join(".tracks")
                .join(dir),
        )?;
    }

    let timestamp = snapshots.get(who).unwrap();

    let new_file_path = fpm::utils::track_path(who, base_path);

    write(whom, *timestamp, &new_file_path).await?;
    println!("{} is now tracking {}", who, whom);

    Ok(())
}

async fn write(whom: &str, timestamp: u128, path: &camino::Utf8PathBuf) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;
    let string = if tokio::fs::metadata(path).await.is_ok() {
        let existing_doc = tokio::fs::read_to_string(path).await?;
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
