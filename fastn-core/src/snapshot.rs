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

// TODO: replace path with config
pub(crate) async fn get_latest_snapshots(
    path: &camino::Utf8PathBuf,
) -> fastn_core::Result<std::collections::BTreeMap<String, u128>> {
    let latest_file_path = path.join(".history/.latest.ftd");
    if !latest_file_path.exists() {
        // TODO: should we error out here?
        return Ok(Default::default());
    }

    let doc = std::fs::read_to_string(&latest_file_path)?;
    resolve_snapshots(&doc).await
}

pub(crate) async fn create_latest_snapshots(
    config: &fastn_core::Config,
    snapshots: &[Snapshot],
) -> fastn_core::Result<()> {
    use tokio::io::AsyncWriteExt;

    let new_file_path = config.latest_ftd();
    let mut snapshot_data = "-- import: fastn".to_string();

    for snapshot in snapshots {
        snapshot_data = format!(
            "{}\n\n-- fastn.snapshot: {}\ntimestamp: {}",
            snapshot_data, snapshot.filename, snapshot.timestamp
        );
    }

    let mut f = tokio::fs::File::create(new_file_path.as_str()).await?;

    f.write_all(snapshot_data.as_bytes()).await?;

    Ok(())
}

pub(crate) fn get_new_version(history: &[&fastn_core::history::FileHistory], file: &str) -> i32 {
    if let Some(file_history) = history.iter().find(|v| v.filename.eq(file)) {
        if let Some(file_edit) = file_history.file_edit.first() {
            return file_edit.version + 1;
        }
    }
    1
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
pub struct Workspace {
    pub filename: String,
    pub base: u128,
    pub conflicted: u128,
    pub workspace: WorkspaceType, // workspace type ours/theirs/conflicted
}

impl Workspace {
    pub(crate) fn is_resolved(&self) -> bool {
        matches!(
            self.workspace,
            WorkspaceType::AbortMerge | WorkspaceType::Revert
        )
    }

    pub(crate) fn set_abort(&mut self) {
        self.workspace = WorkspaceType::AbortMerge;
    }
}

pub(crate) async fn resolve_workspace(
    content: &str,
) -> fastn_core::Result<std::collections::BTreeMap<String, Workspace>> {
    let lib = fastn_core::FastnLibrary::default();
    let b = match fastn_core::doc::parse_ftd("workspace.ftd", content, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse .latest.ftd: {:?}", &e);
            todo!();
        }
    };
    let snapshots: Vec<fastn_core::snapshot::Workspace> = b.get("fastn#workspace")?;
    Ok(snapshots
        .into_iter()
        .map(|v| (v.filename.to_string(), v))
        .collect())
}

pub(crate) async fn create_workspace(
    config: &fastn_core::Config,
    workspaces: &[Workspace],
) -> fastn_core::Result<()> {
    let mut data = vec!["-- import: fastn".to_string()];

    for workspace in workspaces {
        data.push(format!(
            "-- fastn.workspace: {}\nbase: {}\nconflicted: {}\nworkspace: {:?}\n",
            workspace.filename, workspace.base, workspace.conflicted, workspace.workspace
        ));
    }

    fastn_core::utils::update1(
        &config.root.join(".fastn"),
        "workspace.ftd",
        data.join("\n\n").as_bytes(),
    )
    .await?;
    Ok(())
}

pub(crate) async fn get_workspace(
    config: &fastn_core::Config,
) -> fastn_core::Result<std::collections::BTreeMap<String, Workspace>> {
    let latest_file_path = config.root.join(".fastn").join("workspace.ftd");
    if !latest_file_path.exists() {
        // TODO: should we error out here?
        return Ok(Default::default());
    }

    let doc = tokio::fs::read_to_string(&latest_file_path).await?;
    resolve_workspace(&doc).await
}
