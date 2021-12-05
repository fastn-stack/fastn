pub async fn diff() -> fpm::Result<()> {
    let config = fpm::Config::read().await?;
    let snapshots = fpm::snaphot::get_latest_snapshots(config.root.as_str())?;

    for doc in fpm::process_dir(config.root.as_str()).await? {
        if doc.id.starts_with(".history") {
            continue;
        }
        if let Some(diff) = get_diffy(&doc, &snapshots).await? {
            println!("diff: {}", doc.id);
            println!("{}", diff);
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
