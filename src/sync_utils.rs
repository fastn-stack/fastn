use itertools::Itertools;
use sha2::Digest;

#[derive(Clone, PartialEq)]
pub enum Status {
    Conflict,
    NoConflict,
    ClientEditedServerDeleted,
    ClientDeletedServerEdited,
}

impl Status {
    pub(crate) fn is_conflicted(&self) -> bool {
        Status::NoConflict.ne(self)
    }
}

#[derive(Clone)]
pub enum FileStatus {
    Add {
        path: String,
        content: Vec<u8>,
        status: Status,
    },
    Update {
        path: String,
        content: Vec<u8>,
        version: i32,
        status: Status,
    },
    Delete {
        path: String,
        version: i32,
        status: Status,
    },
    Untracked {
        path: String,
        version: i32,
    },
}

impl FileStatus {
    fn is_conflicted(&self) -> bool {
        let status = match self {
            FileStatus::Add { status, .. }
            | FileStatus::Update { status, .. }
            | FileStatus::Delete { status, .. } => status,
            FileStatus::Untracked { .. } => return false,
        };
        status.is_conflicted()
    }

    pub(crate) fn get_file_path(&self) -> String {
        match self {
            FileStatus::Add { path, .. }
            | FileStatus::Update { path, .. }
            | FileStatus::Delete { path, .. }
            | FileStatus::Untracked { path, .. } => path.to_string(),
        }
    }

    pub(crate) fn sync_request(self) -> Option<fpm::apis::sync2::SyncRequestFile> {
        if self.is_conflicted() {
            return None;
        }
        Some(match self {
            FileStatus::Add { path, content, .. } => {
                fpm::apis::sync2::SyncRequestFile::Add { path, content }
            }
            FileStatus::Update {
                path,
                content,
                version,
                ..
            } => fpm::apis::sync2::SyncRequestFile::Update {
                path,
                content,
                version,
            },
            FileStatus::Delete { path, version, .. } => {
                fpm::apis::sync2::SyncRequestFile::Delete { path, version }
            }
            FileStatus::Untracked { .. } => return None,
        })
    }
}

pub(crate) async fn get_files_status(
    config: &fpm::Config,
    workspace: &mut std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry>,
) -> fpm::Result<Vec<FileStatus>> {
    let mut changed_files = get_files_status_wrt_workspace(config, workspace).await?;
    get_files_status_wrt_server_latest(config, &mut changed_files, workspace).await?;
    Ok(changed_files)
}

async fn get_files_status_wrt_workspace(
    config: &fpm::Config,
    workspace: &std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry>,
) -> fpm::Result<Vec<FileStatus>> {
    let files = workspace.values().collect_vec();
    let mut changed_files = vec![];
    for workspace_entry in files {
        let version = if let Some(version) = workspace_entry.version {
            version
        } else {
            let content =
                tokio::fs::read(config.root.join(workspace_entry.filename.as_str())).await?;
            changed_files.push(FileStatus::Add {
                path: workspace_entry.filename.to_string(),
                content,
                status: Status::NoConflict,
            });
            continue;
        };
        if workspace_entry.deleted.unwrap_or(false) {
            changed_files.push(FileStatus::Delete {
                path: workspace_entry.filename.to_string(),
                version: workspace_entry.version.ok_or(fpm::Error::UsageError {
                    message: format!(
                        "{}, which is to be deleted, doesn't define version in workspace",
                        workspace_entry.filename
                    ),
                })?,
                status: Status::NoConflict,
            });
            continue;
        }

        let content = tokio::fs::read(config.root.join(workspace_entry.filename.as_str())).await?;
        let history_path = config.history_path(workspace_entry.filename.as_str(), version);
        let history_content = tokio::fs::read(history_path).await?;
        if sha2::Sha256::digest(&content).eq(&sha2::Sha256::digest(&history_content)) {
            changed_files.push(FileStatus::Untracked {
                path: workspace_entry.filename.to_string(),
                version,
            });
            continue;
        }
        changed_files.push(FileStatus::Update {
            path: workspace_entry.filename.to_string(),
            content,
            version,
            status: Status::NoConflict,
        });
    }
    Ok(changed_files)
}

async fn get_files_status_wrt_server_latest(
    config: &fpm::Config,
    files: &mut Vec<FileStatus>,
    workspace: &mut std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry>,
) -> fpm::Result<()> {
    let mut remove_files = vec![];
    let server_latest = config.get_latest_file_edits_with_deleted().await?;
    for (index, file) in files.iter_mut().enumerate() {
        match file {
            FileStatus::Untracked { .. } => {
                continue;
            }
            FileStatus::Add { path, content, .. } => {
                let server_version = if let Some(file_edit) = server_latest.get(path) {
                    if file_edit.is_deleted() {
                        continue;
                    }
                    file_edit.version
                } else {
                    continue;
                };
                let history_path = config.history_path(path, server_version);
                let history_content = tokio::fs::read(history_path).await?;
                if sha2::Sha256::digest(content).eq(&sha2::Sha256::digest(history_content)) {
                    workspace.insert(
                        path.to_string(),
                        fpm::workspace::WorkspaceEntry {
                            filename: path.to_string(),
                            deleted: None,
                            version: Some(server_version),
                        },
                    );
                }

                remove_files.push(index);
            }
            FileStatus::Update {
                path,
                content,
                version,
                status,
            } => {
                let server_file_edit = if let Some(file_edit) = server_latest.get(path) {
                    file_edit
                } else {
                    continue;
                };

                if server_file_edit.is_deleted() {
                    // Conflict: ClientEditedServerDeleted
                    *status = Status::ClientEditedServerDeleted;
                    continue;
                }

                if server_file_edit.version.eq(version) {
                    continue;
                }

                let ancestor_content = if let Ok(content) =
                    tokio::fs::read_to_string(config.history_path(path, *version)).await
                {
                    content
                } else {
                    // binary file like images, can't resolve conflict
                    *status = Status::Conflict;
                    continue;
                };

                // attempt resolving conflict
                let theirs_content =
                    tokio::fs::read_to_string(config.history_path(path, server_file_edit.version))
                        .await?;
                let ours_content = String::from_utf8(content.clone())?;

                match diffy::MergeOptions::new()
                    .set_conflict_style(diffy::ConflictStyle::Merge)
                    .merge(&ancestor_content, &ours_content, &theirs_content)
                {
                    Ok(data) => {
                        tokio::fs::write(path, &data).await?;
                        *content = data.as_bytes().to_vec();
                        *version = server_file_edit.version;
                    }
                    Err(_) => {
                        // can't resolve conflict, so cannot sync
                        *status = Status::Conflict;
                    }
                }
            }
            FileStatus::Delete {
                path,
                version,
                status,
            } => {
                let server_file_edit = if let Some(server_file_edit) = server_latest.get(path) {
                    server_file_edit
                } else {
                    remove_files.push(index);
                    workspace.remove(path);
                    continue;
                };
                if server_file_edit.is_deleted() {
                    remove_files.push(index);
                    workspace.remove(path);
                    continue;
                }
                if !server_file_edit.version.eq(version) {
                    // Conflict modified by server and deleted by client
                    *status = Status::ClientDeletedServerEdited;
                }
            }
        }
    }
    *files = files
        .iter_mut()
        .enumerate()
        .filter_map(|(k, v)| {
            if !remove_files.contains(&k) {
                Some(v.to_owned())
            } else {
                None
            }
        })
        .collect_vec();
    Ok(())
}
