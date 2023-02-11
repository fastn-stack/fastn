#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Status {
    Conflict(i32),
    NoConflict,
    CloneEditedRemoteDeleted(i32),
    CloneDeletedRemoteEdited(i32),
    CloneAddedRemoteAdded(i32),
}

impl Status {
    pub(crate) fn is_conflicted(&self) -> bool {
        Status::NoConflict.ne(self)
    }
    pub(crate) fn is_clone_edited_remote_deleted(&self) -> bool {
        matches!(self, Status::CloneEditedRemoteDeleted(_))
    }
    pub(crate) fn conflicted_version(&self) -> Option<i32> {
        match self {
            Status::Conflict(version) => Some(*version),
            Status::NoConflict => None,
            Status::CloneEditedRemoteDeleted(version) => Some(*version),
            Status::CloneDeletedRemoteEdited(version) => Some(*version),
            Status::CloneAddedRemoteAdded(version) => Some(*version),
        }
    }
}

#[derive(Debug, Clone)]
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
    Uptodate {
        path: String,
        version: i32,
        content: Vec<u8>,
    },
}

impl FileStatus {
    pub(crate) fn is_conflicted(&self) -> bool {
        let status = match self {
            FileStatus::Add { status, .. }
            | FileStatus::Update { status, .. }
            | FileStatus::Delete { status, .. } => status,
            FileStatus::Uptodate { .. } => return false,
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
            FileStatus::Uptodate { version, .. } => Some(*version),
        }
    }

    pub(crate) fn status(&self) -> Option<&Status> {
        match self {
            FileStatus::Add { status, .. }
            | FileStatus::Update { status, .. }
            | FileStatus::Delete { status, .. } => Some(status),
            FileStatus::Uptodate { .. } => None,
        }
    }

    pub(crate) fn get_file_path(&self) -> String {
        match self {
            FileStatus::Add { path, .. }
            | FileStatus::Update { path, .. }
            | FileStatus::Delete { path, .. }
            | FileStatus::Uptodate { path, .. } => path.to_string(),
        }
    }

    pub(crate) fn sync_request(
        self,
        src_cr: Option<usize>,
    ) -> Option<fastn_core::apis::sync2::SyncRequestFile> {
        if self.is_conflicted() {
            return None;
        }
        Some(match self {
            FileStatus::Add { path, content, .. } => {
                fastn_core::apis::sync2::SyncRequestFile::Add {
                    path,
                    content,
                    src_cr,
                }
            }
            FileStatus::Update {
                path,
                content,
                version,
                ..
            } => fastn_core::apis::sync2::SyncRequestFile::Update {
                path,
                content,
                version,
                src_cr,
            },
            FileStatus::Delete { path, version, .. } => {
                fastn_core::apis::sync2::SyncRequestFile::Delete {
                    path,
                    version,
                    src_cr,
                }
            }
            FileStatus::Uptodate { .. } => return None,
        })
    }
}

impl fastn_core::Config {
    pub(crate) async fn get_files_status(&self) -> fastn_core::Result<Vec<FileStatus>> {
        use itertools::Itertools;

        let mut workspace: std::collections::BTreeMap<
            String,
            fastn_core::workspace::WorkspaceEntry,
        > = self.get_workspace_map().await?;
        let changed_files = self.get_files_status_with_workspace(&mut workspace).await?;
        self.write_workspace(workspace.into_values().collect_vec().as_slice())
            .await?;
        Ok(changed_files)
    }

    pub(crate) async fn get_files_status_with_workspace(
        &self,
        workspace: &mut std::collections::BTreeMap<String, fastn_core::workspace::WorkspaceEntry>,
    ) -> fastn_core::Result<Vec<FileStatus>> {
        use itertools::Itertools;

        let mut changed_files: std::collections::BTreeMap<String, FileStatus> = self
            .get_files_status_wrt_workspace(workspace)
            .await?
            .into_iter()
            .map(|v| (v.get_file_path(), v))
            .collect();
        self.get_files_status_wrt_remote_manifest(&mut changed_files, workspace)
            .await?;
        Ok(changed_files.into_values().collect_vec())
    }

    /*async fn update_workspace_using_cr_track(
        &self,
        cr_workspace: &mut fastn_core::workspace::CRWorkspace,
    ) -> fastn_core::Result<Vec<FileStatus>> {
        let cr_tracking_infos = self.get_cr_tracking_info(cr_workspace.cr).await?;
        for tracking_info in cr_tracking_infos {
            if cr_workspace.workspace.contains_key(&tracking_info.filename) {
                continue;
            }
            cr_workspace.workspace.insert(
                tracking_info.filename,
                tracking_info.into_workspace_entry(cr_workspace.cr),
            )
        }

        Ok(())
    }*/

    async fn get_files_status_wrt_workspace(
        &self,
        workspace: &std::collections::BTreeMap<String, fastn_core::workspace::WorkspaceEntry>,
    ) -> fastn_core::Result<Vec<FileStatus>> {
        use sha2::Digest;

        let mut changed_files = vec![];
        for (filename, workspace_entry) in workspace {
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
                    version: workspace_entry
                        .version
                        .ok_or(fastn_core::Error::UsageError {
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
            let history_path = self.history_path(filename.as_str(), version);
            let history_content = tokio::fs::read(history_path).await?;
            if sha2::Sha256::digest(&content).eq(&sha2::Sha256::digest(&history_content)) {
                changed_files.push(FileStatus::Uptodate {
                    path: workspace_entry.filename.to_string(),
                    version,
                    content,
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

    pub(crate) async fn get_files_status_wrt_remote_manifest(
        &self,
        files: &mut std::collections::BTreeMap<String, FileStatus>,
        workspace: &mut std::collections::BTreeMap<String, fastn_core::workspace::WorkspaceEntry>,
    ) -> fastn_core::Result<()> {
        let remote_manifest = self.get_remote_manifest(true).await?;
        let (already_added_files, already_removed_files) = self
            .get_files_status_wrt_manifest(files, &remote_manifest)
            .await?;
        for file_name in already_removed_files {
            workspace.remove(file_name.as_str());
        }
        for workspace_entry in already_added_files {
            workspace.insert(workspace_entry.filename.to_string(), workspace_entry);
        }
        Ok(())
    }

    async fn get_files_status_wrt_manifest(
        &self,
        files: &mut std::collections::BTreeMap<String, FileStatus>,
        manifest: &std::collections::BTreeMap<String, fastn_core::history::FileEdit>,
    ) -> fastn_core::Result<(Vec<fastn_core::workspace::WorkspaceEntry>, Vec<String>)> {
        use sha2::Digest;

        let mut already_removed_files = vec![];
        let mut already_added_files = vec![];
        let mut uptodate_files = vec![];
        for (filename, file) in files.iter_mut() {
            match file {
                FileStatus::Uptodate { .. } => {
                    continue;
                }
                FileStatus::Add {
                    path,
                    content,
                    status,
                } => {
                    let server_version = if let Some(file_edit) = manifest.get(path) {
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
                        already_added_files.push(fastn_core::workspace::WorkspaceEntry {
                            filename: path.to_string(),
                            deleted: None,
                            version: Some(server_version),
                            cr: None,
                        });
                        uptodate_files.push(filename.clone())
                    } else {
                        *status = Status::CloneAddedRemoteAdded(server_version);
                    }
                }
                FileStatus::Update {
                    path,
                    content,
                    version,
                    status,
                } => {
                    let server_file_edit = if let Some(file_edit) = manifest.get(path) {
                        file_edit
                    } else {
                        continue;
                    };

                    if server_file_edit.is_deleted() {
                        // Conflict: CloneEditedRemoteDeleted
                        *status = Status::CloneEditedRemoteDeleted(server_file_edit.version);
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
                            fastn_core::utils::update(self.root.join(filename), data.as_bytes())
                                .await?;
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
                    let server_file_edit = if let Some(server_file_edit) = manifest.get(path) {
                        server_file_edit
                    } else {
                        already_removed_files.push(filename.clone());
                        uptodate_files.push(filename.clone());
                        continue;
                    };
                    if server_file_edit.is_deleted() {
                        already_removed_files.push(filename.clone());
                        uptodate_files.push(filename.clone());
                        continue;
                    }
                    if !server_file_edit.version.eq(version) {
                        // Conflict modified by server and deleted by client
                        *status = Status::CloneDeletedRemoteEdited(server_file_edit.version);
                    }
                }
            }
        }
        *files = files
            .iter_mut()
            .filter_map(|(k, v)| {
                if !uptodate_files.contains(k) {
                    Some((k.to_owned(), v.to_owned()))
                } else {
                    None
                }
            })
            .collect();
        Ok((already_added_files, already_removed_files))
    }
}
