pub async fn start_tracking(
    config: &fastn::Config,
    source: &str,
    target: &str,
) -> fastn::Result<()> {
    tokio::fs::create_dir_all(config.track_dir()).await?;

    let snapshots = fastn::snapshot::get_latest_snapshots(&config.root).await?;
    check(config.root.as_str(), &snapshots, source, target).await?;
    Ok(())
}

async fn check(
    base_path: &str,
    snapshots: &std::collections::BTreeMap<String, u128>,
    source: &str,
    target: &str,
) -> fastn::Result<()> {
    if !snapshots.contains_key(target) {
        return Err(fastn::Error::UsageError {
            message: format!(
                "{} is not synced yet. suggestion: Run `fastn sync {}` to sync the file",
                target, target
            ),
        });
    }

    let timestamp = if let Some(timestamp) = snapshots.get(source) {
        timestamp
    } else {
        return Err(fastn::Error::UsageError {
            message: format!(
                "{} is not synced yet. suggestion: Run `fastn sync {}` to sync the file",
                source, source
            ),
        });
    };

    // if source is already tracking target, print message and return
    {
        let track_path = fastn::utils::track_path(source, base_path);
        let tracks = fastn::tracker::get_tracks(base_path, &track_path)?;

        if tracks.contains_key(target) {
            println!("{} is already tracking {}", source, target);
            return Ok(());
        }
    }

    if let Some((dir, _)) = source.rsplit_once('/') {
        tokio::fs::create_dir_all(
            camino::Utf8PathBuf::from(base_path)
                .join(".tracks")
                .join(dir),
        )
        .await?;
    }

    let new_file_path = fastn::utils::track_path(source, base_path);

    write(target, *timestamp, &new_file_path).await?;
    println!("{} is now tracking {}", source, target);

    Ok(())
}

async fn write(target: &str, timestamp: u128, path: &camino::Utf8PathBuf) -> fastn::Result<()> {
    use tokio::io::AsyncWriteExt;
    let string = if path.exists() {
        let existing_doc = tokio::fs::read_to_string(path).await?;
        format!(
            "{}\n\n-- fastn.track: {}\nself-timestamp: {}",
            existing_doc, target, timestamp
        )
    } else {
        format!(
            "-- import: fastn\n\n-- fastn.track: {}\nself-timestamp: {}",
            target, timestamp
        )
    };

    let mut f = tokio::fs::File::create(path).await?;
    f.write_all(string.as_bytes()).await?;
    Ok(())
}
