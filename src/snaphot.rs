#[derive(serde::Deserialize, Debug)]
pub struct Snapshot {
    pub file: String,
    pub timestamp: String,
}

impl Snapshot {
    pub fn parse(b: &ftd::p2::Document) -> fpm::Result<Vec<Snapshot>> {
        Ok(b.to_owned().instances::<Snapshot>("fpm#snapshot")?)
    }
}

pub(crate) fn get_latest_snapshots(
    base_path: &str,
) -> fpm::Result<std::collections::BTreeMap<String, String>> {
    let mut snapshots = std::collections::BTreeMap::new();
    let new_file_path = format!("{}/.history/.latest.ftd", base_path);
    if std::fs::metadata(&new_file_path).is_err() {
        return Ok(snapshots);
    }

    let lib = fpm::Library {
        file_name: None,
        markdown_content: None,
    };
    let doc = std::fs::read_to_string(&new_file_path)?;
    let b = match ftd::p2::Document::from(base_path, doc.as_str(), &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", base_path, &e);
            todo!();
        }
    };
    let snapshot_list = Snapshot::parse(&b)?;
    for snapshot in snapshot_list {
        snapshots.insert(snapshot.file, snapshot.timestamp);
    }
    Ok(snapshots)
}

pub(crate) async fn create_latest_snapshots(
    base_path: &str,
    snapshots: &[Snapshot],
) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;

    let new_file_path = format!("{}/.history/.latest.ftd", base_path);
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
