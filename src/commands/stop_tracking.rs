pub async fn stop_tracking(config: &fpm::Config, who: &str, whom: Option<&str>) -> fpm::Result<()> {
    check(who, whom, config.root.as_str()).await?;

    Ok(())
}

async fn check(who: &str, whom: Option<&str>, base_path: &str) -> fpm::Result<()> {
    let file_path = fpm::utils::track_path(who, base_path);
    let mut tracks = fpm::tracker::get_tracks(base_path, &file_path)?;
    if let Some(whom) = whom {
        if tracks.remove(whom).is_some() {
            write(&file_path, &tracks).await?;
            println!("{} is now stop tracking {}", who, whom);
            return Ok(());
        } else {
            eprintln!("Error: {} is not tracking {}", who, whom);
        }
    }

    if !tracks.is_empty() {
        println!(
            "Which file to stop tracking? {} tracks following files",
            who
        );
    } else {
        println!("{} tracks no file", who);
    }
    for track in tracks.keys() {
        println!("{}", track);
    }
    Ok(())
}

async fn write(
    file_path: &camino::Utf8PathBuf,
    tracks: &std::collections::BTreeMap<String, fpm::Track>,
) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;

    let mut f = tokio::fs::File::create(file_path).await?;
    let mut string = "-- import: fpm".to_string();

    for track in tracks.values() {
        string = format!(
            "{}\n\n-- fpm.track: {}\nself-timestamp: {}",
            string, track.filename, track.self_timestamp
        );
        if let Some(ref other_timestamp) = track.other_timestamp {
            string = format!("{}\nother-timestamp: {}", string, other_timestamp);
        }
    }
    f.write_all(string.as_bytes()).await?;
    Ok(())
}
