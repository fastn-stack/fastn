pub async fn diff() -> fpm::Result<()> {
    let config = fpm::Config::read().await?;
    let snapshots = fpm::snaphot::get_latest_snapshots(config.root.as_str())?;

    for doc in fpm::process_dir(config.root.as_str(), &config, fpm::ignore_history()).await? {
        if let fpm::FileFound::FTDDocument(doc) = doc {
            if let Some(diff) = get_diffy(&doc, &snapshots).await? {
                println!("diff: {}", doc.id);
                println!("{}", diff);
            }
            get_track_diff(&doc, &snapshots, config.root.as_str()).await?;
        }
    }
    Ok(())
}

async fn get_diffy(
    doc: &fpm::Document,
    snapshots: &std::collections::BTreeMap<String, String>,
) -> fpm::Result<Option<String>> {
    if let Some(timestamp) = snapshots.get(&doc.id) {
        let path = format!(
            "{}/.history/{}",
            doc.base_path.as_str(),
            doc.id.replace(".ftd", &format!(".{}.ftd", timestamp))
        );

        let existing_doc = tokio::fs::read_to_string(&path).await?;
        if doc.document.eq(&existing_doc) {
            return Ok(None);
        }
        let patch = diffy::create_patch(&existing_doc, &doc.document);
        let diff = diffy::PatchFormatter::new()
            .with_color()
            .fmt_patch(&patch)
            .to_string();
        return Ok(Some(diff));
    }
    Ok(None)
}

async fn get_track_diff(
    doc: &fpm::Document,
    snapshots: &std::collections::BTreeMap<String, String>,
    base_path: &str,
) -> fpm::Result<()> {
    let path = format!(
        "{}/.tracks/{}",
        doc.base_path.as_str(),
        doc.id.replace(".ftd", ".track")
    );
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let tracks = fpm::track_data::get_track(base_path, &path)?;
    for track in tracks.values() {
        if let Some(timestamp) = snapshots.get(&track.document_name) {
            if track.other_timestamp.is_none() {
                continue;
            }
            let now_path = format!(
                "{}/.history/{}",
                doc.base_path.as_str(),
                track
                    .document_name
                    .replace(".ftd", &format!(".{}.ftd", timestamp))
            );
            let then_path = format!(
                "{}/.history/{}",
                doc.base_path.as_str(),
                track.document_name.replace(
                    ".ftd",
                    &format!(".{}.ftd", track.other_timestamp.as_ref().unwrap())
                )
            );
            let now_doc = tokio::fs::read_to_string(&now_path).await?;
            let then_doc = tokio::fs::read_to_string(&then_path).await?;
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
                doc.id,
                track.document_name.replace(
                    ".ftd",
                    &format!(".{}.ftd", track.other_timestamp.as_ref().unwrap())
                ),
                track
                    .document_name
                    .replace(".ftd", &format!(".{}.ftd", timestamp))
            );
            println!("{}", diff);
        }
    }
    Ok(())
}
