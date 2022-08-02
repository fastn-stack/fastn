use itertools::Itertools;

pub async fn resolve_conflict(
    config: &fpm::Config,
    path: &str,
    use_ours: bool,
    use_theirs: bool,
    print: bool,
    revive_it: bool,
    delete_it: bool,
) -> fpm::Result<()> {
    let get_files_status = fpm::sync_utils::get_files_status(config).await?;
    let file_status =
        if let Some(file_status) = get_files_status.iter().find(|v| v.get_file_path().eq(path)) {
            file_status
        } else {
            return Err(fpm::Error::UsageError {
                message: format!("{} not found", path),
            });
        };
    let conflicted_data = get_conflict_data(config, file_status).await?;
    if use_ours {
        let content = conflicted_data
            .ours
            .get_content()
            .ok_or(fpm::Error::UsageError {
                message: format!(
                    "Can't find content, Help: Use `fpm resolve-conflict --delete-it {}`",
                    path
                ),
            })?;
        if conflicted_data.theirs.deleted() {
            return fpm::usage_error(format!(
                "`delete-edit-conflict`, Help: Use `fpm resolve-conflict --revive-it {}`",
                path
            ));
        }
        fpm::utils::update(&config.root.join(path), content).await?;
    } else if use_theirs {
        let content = conflicted_data
            .theirs
            .get_content()
            .ok_or(fpm::Error::UsageError {
                message: format!(
                    "Can't find content, Help: Use `fpm resolve-conflict --delete-it {}`",
                    path
                ),
            })?;
        if conflicted_data.ours.deleted() {
            return fpm::usage_error(format!(
                "`delete-edit-conflict`, Help: Use `fpm resolve-conflict --revive-it {}`",
                path
            ));
        }
        fpm::utils::update(&config.root.join(path), content).await?;
    } else if revive_it {
        let content = conflicted_data
            .ours
            .get_content()
            .or_else(|| conflicted_data.theirs.get_content())
            .ok_or(fpm::Error::UsageError {
                message: format!("Can't find content: `{}`", path),
            })?;
        fpm::utils::update(&config.root.join(path), content).await?;
    } else if delete_it {
        if !(conflicted_data.ours.deleted() || conflicted_data.theirs.deleted()) {
            return fpm::usage_error(format!("{} is not in `delete-edit-conflict`", path));
        }
        if config.root.join(path).exists() {
            tokio::fs::remove_file(config.root.join(path)).await?;
        }
    } else if print {
        let content = conflicted_data
            .marker
            .ok_or(fpm::Error::UsageError {
                message: format!(
                    "Can't find marked content, Help: Use `fpm resolve-conflict --use-ours {}` && `fpm resolve-conflict --use-theirs {}`",
                    path, path
                ),
            })?;
        println!("{}", content);
        return Ok(());
    } else {
        let content = conflicted_data
            .marker
            .ok_or(fpm::Error::UsageError {
                message: format!(
                    "Can't find marked content, Help: Use `fpm resolve-conflict --use-ours {}` && `fpm resolve-conflict --use-theirs {}`",
                    path, path
                ),
            })?;
        let edited = edit::edit(content).map_err(|e| fpm::Error::UsageError {
            message: format!("{}, Help: Use `fpm resolve-conflict --print {}`", e, path,),
        })?;
        fpm::utils::update(&config.root.join(path), edited.as_bytes()).await?;
    }

    mark_resolve(config, file_status, delete_it).await?;
    Ok(())
}

async fn mark_resolve(
    config: &fpm::Config,
    file_status: &fpm::sync_utils::FileStatus,
    delete_it: bool,
) -> fpm::Result<()> {
    let path = file_status.get_file_path();
    let server_version = file_status
        .status()
        .and_then(|v| v.conflicted_version())
        .ok_or(fpm::Error::UsageError {
            message: format!("{} is not in conflict", path),
        })?;
    let mut workspace_map: std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry> =
        config
            .read_workspace()
            .await?
            .iter()
            .map(|v| (v.filename.to_string(), v.clone()))
            .collect();
    if delete_it
        && file_status
            .status()
            .map(|v| v.is_client_deleted_server_edited())
            .unwrap_or(false)
    {
        workspace_map.remove(&path);
    } else {
        let file_workspace_entry = workspace_map.get_mut(&path).ok_or(fpm::Error::UsageError {
            message: format!("Can't find entry in workspace for `{}`", path),
        })?;
        file_workspace_entry.version = Some(server_version);
        file_workspace_entry.deleted = if delete_it { Some(true) } else { None };
    }
    config
        .write_workspace(workspace_map.into_values().collect_vec().as_slice())
        .await
}

enum Content {
    Content(Vec<u8>),
    Deleted,
}

impl Content {
    fn get_content(&self) -> Option<&[u8]> {
        match self {
            Content::Content(content) => Some(content),
            Content::Deleted => None,
        }
    }

    fn deleted(&self) -> bool {
        matches!(self, Content::Deleted)
    }
}

struct ConflictData {
    ours: Content,
    theirs: Content,
    marker: Option<String>,
}

async fn get_conflict_data(
    config: &fpm::Config,
    file_status: &fpm::sync_utils::FileStatus,
) -> fpm::Result<ConflictData> {
    match file_status {
        fpm::sync_utils::FileStatus::Add {
            path,
            content,
            status,
        } => {
            let server_version = if let Some(version) = status.conflicted_version() {
                version
            } else {
                return fpm::usage_error(format!("`{}` is not in conflict state", path));
            };
            let history_path = config.history_path(path, server_version);
            let history_content = tokio::fs::read(history_path).await?;
            /* if let Ok(theirs_string) = String::from_utf8(history_content.to_vec()) {
                let ours_string = String::from_utf8(content.to_vec())?;
                let patch = diffy::create_patch(ours_string.as_str(), theirs_string.as_str());
                let patch_content = if with_formatter {
                    let patch_formatter = diffy::PatchFormatter::new().with_color();
                    patch_formatter.fmt_patch(&patch).to_string()
                } else {
                    path.to_string()
                };
                return Ok(ConflictData {
                    ours: Content::Content(content.to_vec()),
                    theirs: Content::Content(history_content),
                    marker: Some(patch_content),
                });
            }*/

            Ok(ConflictData {
                ours: Content::Content(content.to_vec()),
                theirs: Content::Content(history_content),
                marker: None,
            })
        }
        fpm::sync_utils::FileStatus::Update {
            path,
            content,
            version,
            status,
        } => {
            let server_version = if let Some(version) = status.conflicted_version() {
                version
            } else {
                return fpm::usage_error(format!("`{}` is not in conflict state", path));
            };
            if matches!(
                status,
                fpm::sync_utils::Status::ClientEditedServerDeleted(_)
            ) {
                return Ok(ConflictData {
                    ours: Content::Content(content.to_vec()),
                    theirs: Content::Deleted,
                    marker: None,
                });
            }

            if !matches!(status, fpm::sync_utils::Status::Conflict(_)) {
                return fpm::usage_error(format!(
                    "Expected status of the file is Conflict, found: {:?}",
                    status
                ));
            }
            let theirs_path = config.history_path(path, server_version);
            let theirs_content = tokio::fs::read(theirs_path).await?;
            if let Ok(theirs_string) = String::from_utf8(theirs_content.to_vec()) {
                let ours_string = String::from_utf8(content.to_vec())?;
                let ancestor_path = config.history_path(path, *version);
                let ancestor_content = tokio::fs::read(ancestor_path).await?;
                let ancestor_string = String::from_utf8(ancestor_content)?;
                match diffy::MergeOptions::new()
                    .set_conflict_style(diffy::ConflictStyle::Merge)
                    .merge(&ancestor_string, &ours_string, &theirs_string)
                {
                    Ok(data) => {
                        // Not possible to reach here
                        tokio::fs::write(path, &data).await?;
                        return fpm::usage_error(format!("`{}` already resolved", path));
                    }
                    Err(data) => {
                        return Ok(ConflictData {
                            ours: Content::Content(content.to_vec()),
                            theirs: Content::Content(theirs_content),
                            marker: Some(data),
                        });
                    }
                }
            }
            Ok(ConflictData {
                ours: Content::Content(content.to_vec()),
                theirs: Content::Content(theirs_content),
                marker: None,
            })
        }
        fpm::sync_utils::FileStatus::Delete {
            path,
            version,
            status,
        } => {
            if !matches!(
                status,
                fpm::sync_utils::Status::ClientDeletedServerEdited(_)
            ) {
                return fpm::usage_error(format!(
                    "Expected status of the file is ClientDeletedServerEdited, found: {:?}",
                    status
                ));
            }
            let theirs_path = config.history_path(path, *version);
            let theirs_content = tokio::fs::read(theirs_path).await?;
            Ok(ConflictData {
                theirs: Content::Content(theirs_content),
                ours: Content::Deleted,
                marker: None,
            })
        }
        fpm::sync_utils::FileStatus::Untracked { path, version } => {
            return fpm::usage_error(format!(
                "No changes detected for file `{}` with latest version: `{}`",
                path, version
            ))
        }
    }
}
