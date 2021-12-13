pub async fn diff(config: &fpm::Config) -> fpm::Result<()> {
    let snapshots = fpm::snapshot::get_latest_snapshots(config).await?;

    for doc in fpm::get_documents(config).await? {
        if let fpm::File::Ftd(doc) = doc {
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
    snapshots: &std::collections::BTreeMap<String, u128>,
) -> fpm::Result<Option<String>> {
    if let Some(timestamp) = snapshots.get(&doc.id) {
        let path = format!(
            "{}/.history/{}",
            doc.parent_path.as_str(),
            doc.id.replace(".ftd", &format!(".{}.ftd", timestamp))
        );

        let existing_doc = tokio::fs::read_to_string(&path).await?;
        if doc.content.eq(&existing_doc) {
            return Ok(None);
        }
        let patch = diffy::create_patch(&existing_doc, &doc.content);
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
    snapshots: &std::collections::BTreeMap<String, u128>,
    base_path: &str,
) -> fpm::Result<()> {
    let path = format!(
        "{}/.tracks/{}",
        doc.parent_path.as_str(),
        doc.id.replace(".ftd", ".track")
    );
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let tracks = fpm::tracker::get_tracks(base_path, &path)?;
    for track in tracks.values() {
        if let Some(timestamp) = snapshots.get(&track.document_name) {
            if track.other_timestamp.is_none() {
                continue;
            }
            let now_path = format!(
                "{}/.history/{}",
                doc.parent_path.as_str(),
                track
                    .document_name
                    .replace(".ftd", &format!(".{}.ftd", timestamp))
            );
            let then_path = format!(
                "{}/.history/{}",
                doc.parent_path.as_str(),
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
