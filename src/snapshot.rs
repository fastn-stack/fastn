#[derive(serde::Deserialize, Debug)]
pub struct Snapshot {
    pub file: String,
    pub timestamp: String,
}

pub(crate) async fn get_latest_snapshots(
    config: &fpm::Config,
) -> fpm::Result<std::collections::BTreeMap<String, String>> {
    let mut snapshots = std::collections::BTreeMap::new();
    let latest_file_path = config.latest_ftd();
    if !latest_file_path.exists() {
        // TODO: should we error out here?
        return Ok(snapshots);
    }

    let lib = fpm::Library::default();
    let doc = tokio::fs::read_to_string(&latest_file_path).await?;
    let b = match ftd::p2::Document::from(".latest.ftd", doc.as_str(), &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", latest_file_path, &e);
            todo!();
        }
    };
    let snapshot_list: Vec<fpm::Snapshot> = b.get("fpm#spanshot")?;
    for snapshot in snapshot_list {
        snapshots.insert(snapshot.file, snapshot.timestamp);
    }
    Ok(snapshots)
}

pub(crate) async fn create_latest_snapshots(
    config: &fpm::Config,
    snapshots: &[Snapshot],
) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;

    let new_file_path = config.latest_ftd();
    let mut snapshot_data = "-- import: fpm".to_string();

    for snapshot in snapshots {
        snapshot_data = format!(
            "{}\n\n-- fpm.snapshot: {}\ntimestamp: {}",
            snapshot_data, snapshot.file, snapshot.timestamp
        );
    }

    let mut f = tokio::fs::File::create(new_file_path.as_str()).await?;

    f.write_all(snapshot_data.as_bytes()).await?;

    Ok(())
}
