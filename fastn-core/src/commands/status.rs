pub async fn status(config: &fastn_core::Config, source: Option<&str>) -> fastn_core::Result<()> {
    let snapshots = fastn_core::snapshot::get_latest_snapshots(&config.root).await?;
    let workspaces = fastn_core::snapshot::get_workspace(config).await?;
    match source {
        Some(source) => {
            file_status(
                config.package.name.clone(),
                &config.root,
                source,
                &snapshots,
                &workspaces,
            )
            .await
        }
        None => all_status(config, &snapshots, &workspaces).await,
    }
}

async fn file_status(
    package_name: String,
    base_path: &camino::Utf8PathBuf,
    source: &str,
    snapshots: &std::collections::BTreeMap<String, u128>,
    workspaces: &std::collections::BTreeMap<String, fastn_core::snapshot::Workspace>,
) -> fastn_core::Result<()> {
    let path = base_path.join(source);
    if !path.exists() {
        if snapshots.contains_key(source) {
            println!("{:?}: {}", FileStatus::Deleted, source);
        } else {
            eprintln!("Error: {} does not exists", source);
        }
        return Ok(());
    }

    let file = fastn_core::get_file(package_name, &path, base_path).await?;

    let file_status = get_file_status(&file, snapshots, workspaces).await?;
    let track_status = get_track_status(&file, snapshots, base_path.as_str())?;

    let mut clean = true;
    if !file_status.eq(&FileStatus::Uptodate) {
        println!("{:?}: {}", file_status, source);
        clean = false;
    }
    for (i, j) in track_status {
        println!("{}: {} -> {}", j.to_string(), source, i);
        clean = false;
    }
    if clean {
        println!("Nothing to sync, clean working tree");
    }
    Ok(())
}

async fn all_status(
    config: &fastn_core::Config,
    snapshots: &std::collections::BTreeMap<String, u128>,
    workspaces: &std::collections::BTreeMap<String, fastn_core::snapshot::Workspace>,
) -> fastn_core::Result<()> {
    use itertools::Itertools;

    let mut file_status = std::collections::BTreeMap::new();
    let mut track_status = std::collections::BTreeMap::new();
    for doc in config.get_files(&config.package).await? {
        let status = get_file_status(&doc, snapshots, workspaces).await?;
        let track = get_track_status(&doc, snapshots, config.root.as_str())?;
        if !track.is_empty() {
            track_status.insert(doc.get_id(), track);
        }
        file_status.insert(doc.get_id(), status);
    }

    let deleted_files = snapshots
        .keys()
        .filter(|v| !file_status.contains_key(v.as_str()))
        .collect_vec();

    file_status.extend(
        deleted_files
            .into_iter()
            .map(|v| (v.to_string(), FileStatus::Deleted)),
    );

    let clean_file_status = print_file_status(&file_status);
    let clean_track_status = print_track_status(&track_status);
    if !clean_file_status && clean_track_status {
        println!("Nothing to sync, clean working tree");
    }
    Ok(())
}

pub(crate) async fn get_file_status(
    doc: &fastn_core::File,
    snapshots: &std::collections::BTreeMap<String, u128>,
    workspaces: &std::collections::BTreeMap<String, fastn_core::snapshot::Workspace>,
) -> fastn_core::Result<FileStatus> {
    use sha2::Digest;
    // Added
    // 1. if file do not present in `latest.ftd`
    // NP: If server deleted and client modified so in that case `latest.ftd` won't contain entry
    // For that in that entry will be present in workspace.ftd
    // Added: If file do not present in `latest.ftd` and also do not present in .fastn/workspace.ftd

    if let Some(workspace) = workspaces.get(&doc.get_id()) {
        return Ok(match workspace.workspace {
            fastn_core::snapshot::WorkspaceType::Conflicted
            | fastn_core::snapshot::WorkspaceType::Revert
            | fastn_core::snapshot::WorkspaceType::AbortMerge => {
                let conflicted_version = workspace.conflicted;
                match snapshots.get(&doc.get_id()) {
                    Some(latest_version) if conflicted_version.lt(latest_version) => {
                        FileStatus::Outdated
                    }
                    _ => FileStatus::Conflicted,
                }
            }
            fastn_core::snapshot::WorkspaceType::CloneEditedRemoteDeleted => {
                FileStatus::CloneEditedRemoteDeleted
            }
            fastn_core::snapshot::WorkspaceType::CloneDeletedRemoteEdited => {
                FileStatus::CloneDeletedRemoteEdited
            }
        });
    }

    if let Some(timestamp) = snapshots.get(&doc.get_id()) {
        let path = fastn_core::utils::history_path(&doc.get_id(), &doc.get_base_path(), timestamp);

        let content = tokio::fs::read(&doc.get_full_path()).await?;
        let existing_doc = tokio::fs::read(&path).await?;
        if sha2::Sha256::digest(content).eq(&sha2::Sha256::digest(existing_doc)) {
            return Ok(FileStatus::Uptodate);
        }
        return Ok(FileStatus::Modified);
    }
    Ok(FileStatus::Added)
}

fn get_track_status(
    doc: &fastn_core::File,
    snapshots: &std::collections::BTreeMap<String, u128>,
    base_path: &str,
) -> fastn_core::Result<std::collections::BTreeMap<String, TrackStatus>> {
    let path = fastn_core::utils::track_path(&doc.get_id(), &doc.get_base_path());
    let mut track_list = std::collections::BTreeMap::new();
    if !path.exists() {
        return Ok(track_list);
    }
    let tracks = fastn_core::tracker::get_tracks(base_path, &path)?;
    for track in tracks.values() {
        // ignore in case of the translation package
        if doc.get_id().eq(track.filename.as_str()) && track.last_merged_version.is_some() {
            continue;
        }
        if !snapshots.contains_key(&track.filename) {
            eprintln!(
                "Error: {} is tracked by {}, but {} is either removed or never synced",
                track.filename,
                doc.get_id(),
                track.filename
            );
            continue;
        }
        let timestamp = snapshots.get(&track.filename).unwrap();
        let track_status = if track.other_timestamp.is_none() {
            TrackStatus::NeverMarked
        } else if timestamp.eq(track.other_timestamp.as_ref().unwrap()) {
            TrackStatus::UptoDate
        } else {
            let now = *timestamp;
            let then = track.other_timestamp.as_ref().unwrap();
            let diff = std::time::Duration::from_nanos((now - then) as u64);
            TrackStatus::OutOfDate {
                seconds: diff.as_secs(),
            }
        };
        track_list.insert(track.filename.to_string(), track_status);
    }
    Ok(track_list)
}

fn print_track_status(
    track_status: &std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<String, TrackStatus>,
    >,
) -> bool {
    let mut status = true;
    for (k, v) in track_status {
        for (i, j) in v {
            if j.eq(&TrackStatus::UptoDate) {
                continue;
            }
            println!("{}: {} -> {}", j.to_string(), k, i);
            status = false;
        }
    }
    status
}

fn print_file_status(file_status: &std::collections::BTreeMap<String, FileStatus>) -> bool {
    let mut any_modification = false;

    for (id, status) in file_status
        .iter()
        .filter(|(_, f)| !f.eq(&&FileStatus::Uptodate))
    {
        any_modification = true;
        println!("{:?}: {}", status, id);
    }

    any_modification
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FileStatus {
    Modified,
    Added,
    Deleted,
    Uptodate,
    Conflicted,
    Outdated,
    CloneEditedRemoteDeleted,
    CloneDeletedRemoteEdited,
}

#[derive(Debug, PartialEq, Eq)]
enum TrackStatus {
    UptoDate,
    NeverMarked,
    OutOfDate { seconds: u64 },
}

impl ToString for TrackStatus {
    fn to_string(&self) -> String {
        match self {
            TrackStatus::UptoDate => "Up to date".to_string(),
            TrackStatus::NeverMarked => "Never marked".to_string(),
            TrackStatus::OutOfDate { seconds } => {
                format!("Out of date({})", fastn_core::utils::seconds_to_human(*seconds))
            }
        }
    }
}
