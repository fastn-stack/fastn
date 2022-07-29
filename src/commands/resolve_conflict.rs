pub async fn resolve_conflict(
    config: &fpm::Config,
    path: &str,
    use_ours: bool,
    use_theirs: bool,
) -> fpm::Result<()> {
    let new_content = fpm::editor::editor(
        "Hello World\nThis is Arpita\n\
        Unlike other package managers like\npypi, npm and crates, there is no 
        central package repository in FPM. 
         \nSince every FPM package is a website, that website acts as the package repository.\n\n
        What this means is when fpm sees fifthtry.github.io/doc-site as a dependency, \n
        it fetches the content of fifthtry.github.io/doc-site/FPM.ftd file which acts \n
        as the meta data for the package, and the meta data includes the URL from \n
        where the package ZIP can be downloaded.\n\n\
        \
        In our examples we use Github’s zip serving feature to let Github generate \n
        and distribute our .zip file. If you are not using Github, you can store \n
        the zip file containing entire package in some other location, like S3, \n
        or a server you control, and fpm will work with that.",
        Some(std::path::PathBuf::from("index.ftd")),
    )?;
    dbg!(&new_content);
    let get_files_status = fpm::sync_utils::get_files_status(config).await?;
    let file_status =
        if let Some(file_status) = get_files_status.iter().find(|v| v.get_file_path().eq(path)) {
            file_status
        } else {
            return Err(fpm::Error::UsageError {
                message: format!("{} not found", path),
            });
        };

    Ok(())
}

enum Content {
    Content(Vec<u8>),
    Deleted,
}

struct ConflictData {
    ours: Content,
    theirs: Content,
    marker: Option<String>,
}

/*fn get_conflict_data(
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
            return Ok(ConflictData {
                ours: Content::Content(content.to_vec()),
                theirs: Content::Content(theirs_content),
                marker: None,
            });
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
            return Ok(ConflictData {
                theirs: Content::Content(theirs_content),
                ours: Content::Deleted,
                marker: None,
            });
        }
        fpm::sync_utils::FileStatus::Untracked { path, version } => {
            return fpm::usage_error(format!(
                "No changes detected for file `{}` with latest version: `{}`",
                path, version
            ))
        }
    }
    Ok(())
}*/
