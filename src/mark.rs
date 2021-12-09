pub async fn mark(who: &str, whom: &str) -> fpm::Result<()> {
    let config = fpm::Config::read().await?;
    let who = format!("{}.ftd", who);
    let whom = format!("{}.ftd", whom);

    let snapshots = fpm::snaphot::get_latest_snapshots(config.root.as_str())?;

    check(&snapshots, &who, &whom, config.root.as_str()).await?;

    Ok(())
}

async fn check(
    snapshots: &std::collections::BTreeMap<String, String>,
    who: &str,
    whom: &str,
    base_path: &str,
) -> fpm::Result<()> {
    let timestamp = snapshots.get(whom).unwrap();

    let file_path = format!("{}/.tracks/{}", base_path, who.replace(".ftd", ".track"));
    let mut tracks = fpm::track_data::get_track(base_path, &file_path)?;
    if let Some(track) = tracks.get_mut(whom) {
        track.other_timestamp = Some(timestamp.to_string());
    }
    write(&file_path, &tracks).await?;
    Ok(())
}
async fn write(
    file_path: &str,
    tracks: &std::collections::BTreeMap<String, fpm::Tracks>,
) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;

    let mut f = tokio::fs::File::create(file_path).await?;
    let mut string = "-- import: fpm".to_string();

    for track in tracks.values() {
        string = format!(
            "{}\n\n-- fpm.track: {}\nself-timestamp: {}",
            string, track.document_name, track.self_timestamp
        );
        if let Some(ref other_timestamp) = track.other_timestamp {
            string = format!("{}\nother-timestamp: {}", string, other_timestamp);
        }
    }
    f.write_all(string.as_bytes()).await?;
    Ok(())
}
