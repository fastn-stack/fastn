pub async fn diff(
    config: &fastn_core::Config,
    files: Option<Vec<String>>,
    all: bool,
) -> fastn_core::Result<()> {
    let snapshots =
        fastn_core::snapshot::get_latest_snapshots(&config.ds, config.ds.root()).await?;
    let all = all || files.is_some();
    let documents = if let Some(ref files) = files {
        let files = files
            .iter()
            .map(|x| config.ds.root().join(x))
            .collect::<Vec<fastn_ds::Path>>();
        fastn_core::paths_to_files(
            &config.ds,
            config.package.name.as_str(),
            files,
            &config.ds.root(),
        )
        .await?
    } else {
        config.get_files(&config.package).await?
    };
    for doc in documents {
        if let Some(diff) = get_diffy(config, &doc, &snapshots).await? {
            println!("diff: {}", doc.get_id());
            println!("{}", diff);
        }
        if all {
            get_track_diff(config, &doc, &snapshots, config.ds.root()).await?;
        }
    }
    Ok(())
}

async fn get_diffy(
    config: &fastn_core::Config,
    doc: &fastn_core::File,
    snapshots: &std::collections::BTreeMap<String, u128>,
) -> fastn_core::Result<Option<String>> {
    if let Some(timestamp) = snapshots.get(doc.get_id()) {
        let path = fastn_core::utils::history_path(
            doc.get_id(),
            &fastn_ds::Path::new(doc.get_base_path()),
            timestamp,
        );
        let content = config.ds.read_to_string(&doc.get_full_path()).await?;

        let existing_doc = config.ds.read_to_string(&path).await?;
        if content.eq(&existing_doc) {
            return Ok(None);
        }
        let patch = diffy::create_patch(&existing_doc, &content);
        let diff = diffy::PatchFormatter::new()
            .with_color()
            .fmt_patch(&patch)
            .to_string();
        return Ok(Some(diff));
    }
    Ok(None)
}

async fn get_track_diff(
    config: &fastn_core::Config,
    doc: &fastn_core::File,
    snapshots: &std::collections::BTreeMap<String, u128>,
    base_path: &fastn_ds::Path,
) -> fastn_core::Result<()> {
    let path =
        fastn_core::utils::track_path(doc.get_id(), &fastn_ds::Path::new(doc.get_base_path()));
    let tracks = fastn_core::tracker::get_tracks(config, base_path, &path).await?;
    for track in tracks.values() {
        if let Some(timestamp) = snapshots.get(&track.filename) {
            if track.other_timestamp.is_none() {
                continue;
            }
            let now_path = fastn_core::utils::history_path(
                &track.filename,
                &fastn_ds::Path::new(doc.get_base_path()),
                timestamp,
            );

            let then_path = fastn_core::utils::history_path(
                &track.filename,
                &fastn_ds::Path::new(doc.get_base_path()),
                track.other_timestamp.as_ref().unwrap(),
            );

            let now_doc = config.ds.read_to_string(&now_path).await?;
            let then_doc = config.ds.read_to_string(&then_path).await?;
            if now_doc.eq(&then_doc) {
                continue;
            }
            let patch = diffy::create_patch(&then_doc, &now_doc);
            let diff = diffy::PatchFormatter::new()
                .with_color()
                .fmt_patch(&patch)
                .to_string();
            println!(
                "diff {} -> {}: {}",
                doc.get_id(),
                then_path
                    .to_string()
                    .replace(&format!("{}/.history/", doc.get_base_path()), ""),
                now_path
                    .to_string()
                    .replace(&format!("{}/.history/", doc.get_base_path()), ""),
            );
            println!("{}", diff);
        }
    }
    Ok(())
}
