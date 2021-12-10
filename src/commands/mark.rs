pub async fn mark(who: &str, whom: Option<&str>) -> fpm::Result<()> {
    let config = fpm::Config::read().await?;

    let snapshots = fpm::snapshot::get_latest_snapshots(config.root.as_str())?;

    check(&snapshots, who, whom, config.root.as_str()).await?;

    Ok(())
}

async fn check(
    snapshots: &std::collections::BTreeMap<String, String>,
    who: &str,
    whom: Option<&str>,
    base_path: &str,
) -> fpm::Result<()> {
    let file_path = format!("{}/.tracks/{}", base_path, who.replace(".ftd", ".track"));
    let mut tracks = fpm::tracker::get_tracks(base_path, &file_path)?;
    if let Some(whom) = whom {
        if let Some(track) = tracks.get_mut(whom) {
            if let Some(timestamp) = snapshots.get(whom) {
                track.other_timestamp = Some(timestamp.to_string());
                write(&file_path, &tracks).await?;
                println!("{} is now marked upto date with {}", who, whom);
                return Ok(());
            } else {
                eprintln!("Error: {} is removed. Can't mark {} upto date", whom, who);
            }
        } else {
            eprintln!("Error: {} is not tracking {}", who, whom);
        }
    }
    println!("Which file to mark? {} tracks following files", who);
    for track in tracks.keys() {
        println!("{}", track);
    }
    Ok(())
}
async fn write(
    file_path: &str,
    tracks: &std::collections::BTreeMap<String, fpm::Track>,
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
