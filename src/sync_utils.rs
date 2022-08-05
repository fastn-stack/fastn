use itertools::Itertools;
use sha2::Digest;

#[derive(Clone, PartialEq, Debug)]
pub enum Status {
    Conflict(i32),
    NoConflict,
    ClientEditedServerDeleted(i32),
    ClientDeletedServerEdited(i32),
}

impl Status {
    pub(crate) fn is_conflicted(&self) -> bool {
        Status::NoConflict.ne(self)
    }
    pub(crate) fn is_client_edited_server_deleted(&self) -> bool {
        matches!(self, Status::ClientEditedServerDeleted(_))
    }
    pub(crate) fn conflicted_version(&self) -> Option<i32> {
        match self {
            Status::Conflict(version) => Some(*version),
            Status::NoConflict => None,
            Status::ClientEditedServerDeleted(version) => Some(*version),
            Status::ClientDeletedServerEdited(version) => Some(*version),
        }
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

    pub(crate) fn get_latest_version(&self) -> Option<i32> {
        // Either file must be in conflict with latest version, so conflicted version would be
        // latest, or it's version would be latest
        match self {
            FileStatus::Add { .. } => None,
            FileStatus::Update {
                status, version, ..
            }
            | FileStatus::Delete {
                status, version, ..
            } => Some(status.conflicted_version().unwrap_or(*version)),
            FileStatus::Untracked { version, .. } => Some(*version),
        }
    }

    pub(crate) fn status(&self) -> Option<&Status> {
        match self {
            FileStatus::Add { status, .. }
            | FileStatus::Update { status, .. }
            | FileStatus::Delete { status, .. } => Some(status),
            FileStatus::Untracked { .. } => None,
        }
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

impl fpm::Config {
    pub(crate) async fn get_files_status(&self) -> fpm::Result<Vec<FileStatus>> {
        let file_list = self.read_workspace().await?;
        let mut workspace: std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry> =
            file_list
                .iter()
                .map(|v| (v.filename.to_string(), v.clone()))
                .collect();
        let changed_files = self.get_files_status_with_workspace(&mut workspace).await?;
        self.write_workspace(workspace.into_values().collect_vec().as_slice())
            .await?;
        Ok(changed_files)
    }

    pub(crate) async fn get_files_status_with_workspace(
        &self,
        workspace: &mut std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry>,
    ) -> fpm::Result<Vec<FileStatus>> {
        let mut changed_files = self.get_files_status_wrt_workspace(workspace).await?;
        self.get_files_status_wrt_server_latest(&mut changed_files, workspace)
            .await?;
        Ok(changed_files)
    }

    async fn get_files_status_wrt_workspace(
        &self,
        workspace: &std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry>,
    ) -> fpm::Result<Vec<FileStatus>> {
        let files = workspace.values().collect_vec();
        let mut changed_files = vec![];
        for workspace_entry in files {
            let version = if let Some(version) = workspace_entry.version {
                version
            } else {
                let content =
                    tokio::fs::read(self.root.join(workspace_entry.filename.as_str())).await?;
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

            let content =
                tokio::fs::read(self.root.join(workspace_entry.filename.as_str())).await?;
            let history_path = self.history_path(workspace_entry.filename.as_str(), version);
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
        &self,
        files: &mut Vec<FileStatus>,
        workspace: &mut std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry>,
    ) -> fpm::Result<()> {
        let mut remove_files = vec![];
        let server_latest = self.get_latest_file_edits().await?;
        for (index, file) in files.iter_mut().enumerate() {
            match file {
                FileStatus::Untracked { .. } => {
                    continue;
                }
                FileStatus::Add {
                    path,
                    content,
                    status,
                } => {
                    let server_version = if let Some(file_edit) = server_latest.get(path) {
                        if file_edit.is_deleted() {
                            continue;
                        }
                        file_edit.version
                    } else {
                        continue;
                    };
                    let history_path = self.history_path(path, server_version);
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
                        remove_files.push(index);
                    } else {
                        *status = Status::Conflict(server_version);
                    }
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
                        *status = Status::ClientEditedServerDeleted(server_file_edit.version);
                        continue;
                    }

                    if server_file_edit.version.eq(version) {
                        continue;
                    }

                    let ancestor_content = if let Ok(content) =
                        tokio::fs::read_to_string(self.history_path(path, *version)).await
                    {
                        content
                    } else {
                        // binary file like images, can't resolve conflict
                        *status = Status::Conflict(server_file_edit.version);
                        continue;
                    };

                    // attempt resolving conflict
                    let theirs_content = tokio::fs::read_to_string(
                        self.history_path(path, server_file_edit.version),
                    )
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
                            *status = Status::Conflict(server_file_edit.version);
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
                        *status = Status::ClientDeletedServerEdited(server_file_edit.version);
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
}
