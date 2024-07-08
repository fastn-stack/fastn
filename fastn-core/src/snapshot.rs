#[derive(serde::Deserialize, Debug)]
pub struct Snapshot {
    pub filename: String, // relative file name with respect to package root
    pub timestamp: u128,
}

pub(crate) async fn resolve_snapshots(
    content: &str,
) -> fastn_core::Result<std::collections::BTreeMap<String, u128>> {
    if content.trim().is_empty() {
        return Ok(Default::default());
    }
    let lib = fastn_core::FastnLibrary::default();
    let b = match fastn_core::doc::parse_ftd(".latest.ftd", content, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse .latest.ftd: {:?}", &e);
            todo!();
        }
    };
    let snapshots: Vec<fastn_core::Snapshot> = b.get("fastn#snapshot")?;
    Ok(snapshots
        .into_iter()
        .map(|v| (v.filename, v.timestamp))
        .collect())
}

pub(crate) async fn get_latest_snapshots(
    ds: &fastn_ds::DocumentStore,
    path: &fastn_ds::Path,
    session_id: &Option<String>,
) -> fastn_core::Result<std::collections::BTreeMap<String, u128>> {
    let latest_file_path = path.join(".history/.latest.ftd");
    if !ds.exists(&latest_file_path).await {
        // TODO: should we error out here?
        return Ok(Default::default());
    }

    let doc = ds.read_to_string(&latest_file_path, session_id).await?;
    resolve_snapshots(&doc).await
}

/// Related to workspace
#[derive(PartialEq, Eq, Debug, serde::Deserialize)]
pub enum WorkspaceType {
    AbortMerge,
    Revert,
    Conflicted,
    CloneEditedRemoteDeleted,
    CloneDeletedRemoteEdited,
}

#[derive(serde::Deserialize, Debug)]
pub(crate) struct Workspace {}
