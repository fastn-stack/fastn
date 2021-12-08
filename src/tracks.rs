pub async fn tracks(who: &str, whom: &str) -> fpm::Result<()> {
    let config = fpm::Config::read().await?;
    let who = format!("{}.ftd", who);
    let whom = format!("{}.ftd", whom);

    tokio::fs::create_dir_all(format!("{}/.tracks", config.root.as_str()).as_str()).await?;

    let snapshots = fpm::snaphot::get_latest_snapshots(config.root.as_str())?;

    for doc in fpm::process_dir(config.root.as_str(), &config).await? {
        check(&doc, &snapshots, &who, &whom).await?;
    }
    Ok(())
}

async fn check(
    doc: &fpm::Document,
    snapshots: &std::collections::BTreeMap<String, String>,
    who: &str,
    whom: &str,
) -> fpm::Result<()> {
    if doc.id.contains('/') {
        let (dir, _) = doc.id.rsplit_once('/').unwrap();
        std::fs::create_dir_all(format!("{}/.tracks/{}", doc.base_path.as_str(), dir))?;
    }

    if !doc.id.eq(who) {
        return Ok(());
    }

    let timestamp = snapshots.get(who).unwrap();

    let new_file_path = format!(
        "{}/.tracks/{}",
        doc.base_path.as_str(),
        doc.id.replace(".ftd", ".track")
    );

    write(whom, timestamp, &new_file_path).await?;

    Ok(())
}

async fn write(whom: &str, timestamp: &str, path: &str) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;
    let string = if tokio::fs::metadata(path).await.is_ok() {
        let existing_doc = tokio::fs::read_to_string(&path).await?;
        format!(
            "{}\n\n-- fpm.track: {}\nself-timestamp: {}
            ",
            existing_doc, whom, timestamp
        )
    } else {
        format!(
            "-- import: fpm\n\n-- fpm.track: {}\nself-timestamp: {}
            ",
            whom, timestamp
        )
    };

    let mut f = tokio::fs::File::create(path).await?;
    f.write_all(string.as_bytes()).await?;
    Ok(())
}
