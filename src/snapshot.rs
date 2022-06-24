#[derive(serde::Deserialize, Debug)]
pub struct Snapshot {
    pub filename: String, // relative file name with respect to package root
    pub timestamp: u128,
}

pub(crate) async fn resolve_snapshots(
    content: &str,
) -> fpm::Result<std::collections::BTreeMap<String, u128>> {
    let lib = fpm::FPMLibrary::default();
    let b = match fpm::doc::parse_ftd(".latest.ftd", content, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse .latest.ftd: {:?}", &e);
            todo!();
        }
    };
    let snapshots: Vec<fpm::Snapshot> = b.get("fpm#snapshot")?;
    Ok(snapshots
        .into_iter()
        .map(|v| (v.filename, v.timestamp))
        .collect())
}

pub(crate) async fn get_latest_snapshots(
    path: &camino::Utf8PathBuf,
) -> fpm::Result<std::collections::BTreeMap<String, u128>> {
    let latest_file_path = path.join(".history/.latest.ftd");
    if !latest_file_path.exists() {
        // TODO: should we error out here?
        return Ok(Default::default());
    }

    let doc = std::fs::read_to_string(&latest_file_path)?;
    resolve_snapshots(&doc).await
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
            snapshot_data, snapshot.filename, snapshot.timestamp
        );
    }

    let mut f = tokio::fs::File::create(new_file_path.as_str()).await?;

    f.write_all(snapshot_data.as_bytes()).await?;

    Ok(())
}
