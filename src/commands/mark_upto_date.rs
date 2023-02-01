pub async fn mark_upto_date(
    config: &fastn::Config,
    who: &str,
    whom: Option<&str>,
) -> fastn::Result<()> {
    match config.package.translation_of.as_ref() {
        Some(ref original) => mark_upto_date_translation(who, whom, config, original).await,
        _ => mark_upto_date_simple(who, whom, config).await,
    }
}

async fn mark_upto_date_translation(
    who: &str,
    whom: Option<&str>,
    config: &fastn::Config,
    original_package: &fastn::Package,
) -> fastn::Result<()> {
    if let Some(whom) = whom {
        return Err(fastn::Error::UsageError {
            message: format!(
                "Cannot give target for translation package. suggestion: Remove `--target {}`",
                whom
            ),
        });
    }
    let file_path = fastn::utils::track_path(who, config.root.as_str());
    let mut tracks = fastn::tracker::get_tracks(config.root.as_str(), &file_path)?;

    let original_snapshot = fastn::snapshot::get_latest_snapshots(&config.original_path()?).await?;
    let original_timestamp = match original_snapshot.get(who) {
        Some(timestamp) => timestamp,
        _ => {
            return Err(fastn::Error::UsageError {
                message: format!("{} is not present in the original package.", who),
            })
        }
    };
    if let Some(track) = tracks.get_mut(who) {
        track.last_merged_version = Some(*original_timestamp);
    } else {
        let snapshots = fastn::snapshot::get_latest_snapshots(&config.root).await?;
        let self_timestamp = match snapshots.get(who) {
            Some(timestamp) => timestamp,
            _ => {
                return Err(fastn::Error::UsageError {
                    message: format!(
                        "{} is not synced yet. suggestion: Run `fastn sync {}` to sync the file",
                        who, who
                    ),
                })
            }
        };
        tracks.insert(
            who.to_string(),
            fastn::Track {
                filename: who.to_string(),
                package: Some(original_package.name.clone()),
                version: None,
                other_timestamp: None,
                self_timestamp: *self_timestamp,
                last_merged_version: Some(*original_timestamp),
            },
        );
    }
    println!("{} is now marked upto date", who);
    write(&file_path, &tracks).await
}

async fn mark_upto_date_simple(
    who: &str,
    whom: Option<&str>,
    config: &fastn::Config,
) -> fastn::Result<()> {
    let file_path = fastn::utils::track_path(who, config.root.as_str());
    let mut tracks = fastn::tracker::get_tracks(config.root.as_str(), &file_path)?;
    if let Some(whom) = whom {
        return if let Some(track) = tracks.get_mut(whom) {
            let snapshots = fastn::snapshot::get_latest_snapshots(&config.root).await?;
            if let Some(timestamp) = snapshots.get(whom) {
                track.other_timestamp = Some(*timestamp);
                write(&file_path, &tracks).await?;
                println!("{} is now marked upto date with {}", who, whom);
                Ok(())
            } else {
                fastn::usage_error(format!(
                    "Error: {} is removed. Can't mark {} upto date",
                    whom, who
                ))
            }
        } else {
            fastn::usage_error(format!("Error: {} is not tracking {}", who, whom))
        };
    }

    if tracks.is_empty() {
        println!("{} tracks no file", who);
    } else {
        println!("Which file to mark? {} tracks following files:", who);
        for track in tracks.keys() {
            println!("{}", track);
        }
    }

    Ok(())
}

async fn write(
    file_path: &camino::Utf8PathBuf,
    tracks: &std::collections::BTreeMap<String, fastn::Track>,
) -> fastn::Result<()> {
    use tokio::io::AsyncWriteExt;
    if let Some((dir, _)) = file_path.as_str().rsplit_once('/') {
        tokio::fs::create_dir_all(dir).await?;
    }
    let mut f = tokio::fs::File::create(file_path).await?;
    let mut string = "-- import: fastn".to_string();

    for track in tracks.values() {
        string = format!(
            "{}\n\n-- fastn.track: {}\nself-timestamp: {}",
            string, track.filename, track.self_timestamp
        );
        if let Some(ref other_timestamp) = track.other_timestamp {
            string = format!("{}\nother-timestamp: {}", string, other_timestamp);
        }
        if let Some(ref last_merged_version) = track.last_merged_version {
            string = format!("{}\nlast-merged-version: {}", string, last_merged_version);
        }
        if let Some(ref package) = track.package {
            string = format!("{}\npackage: {}", string, package);
        }
    }
    f.write_all(string.as_bytes()).await?;
    Ok(())
}
