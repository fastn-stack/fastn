use itertools::Itertools;
use sha2::Digest;

pub async fn merge(
    config: &fpm::Config,
    src: Option<&str>,
    dest: &str,
    file: Option<&str>,
) -> fpm::Result<()> {
    let src = src.unwrap_or("main");

    if src.eq("main") {
        let dest = dest.parse::<usize>()?;
        merge_main_into_cr(config, dest, file).await?;
    } else if dest.eq("main") {
        let src = src.parse::<usize>()?;
        merge_cr_into_main(config, src, file).await?;
    } else {
        let src = src.parse::<usize>()?;
        let dest = dest.parse::<usize>()?;
        merge_cr_into_cr(config, src, dest, file).await?;
    }

    Ok(())
}

async fn merge_cr_into_cr(
    _config: &fpm::Config,
    _src: usize,
    _dest: usize,
    _file: Option<&str>,
) -> fpm::Result<()> {
    unimplemented!()
}

async fn merge_main_into_cr(
    config: &fpm::Config,
    dest: usize,
    _file: Option<&str>,
) -> fpm::Result<()> {
    let cr_status = config
        .get_cr_status(dest)
        .await?
        .into_iter()
        .filter(|v| v.status().is_some())
        .collect_vec();
    if !cr_status.is_empty() {
        cr_status
            .iter()
            .map(|v| fpm::commands::sync_status::print_status(v, false))
            .collect_vec();
        return fpm::usage_error("Help: Use `fpm sync`".to_string());
    }

    let remote_manifest: std::collections::BTreeMap<String, fpm::history::FileEdit> = config
        .get_remote_manifest(true)
        .await?
        .into_iter()
        .filter(|(k, _)| !(k.starts_with("-/") || k.starts_with(".tracks/")))
        .collect();
    let cr_manifest = config.get_cr_manifest(dest).await?;
    let (cr_track_manifest, cr_file_manifest) = cr_manifest.into_iter().fold(
        (
            std::collections::HashMap::new(),
            std::collections::HashMap::new(),
        ),
        |(mut cr_track_status, mut cr_file_status), (file, file_edit)| {
            if file.starts_with(".tracks/") {
                cr_track_status.insert(file, file_edit);
            } else {
                cr_file_status.insert(file, file_edit);
            }
            (cr_track_status, cr_file_status)
        },
    );
    let mut conflicted_file_status = vec![];
    let deleted_files_path = config.cr_deleted_file_path(dest);
    let deleted_file_str = config.path_without_root(&deleted_files_path)?;
    let mut workspace = config.get_workspace_map().await?;
    for (cr_file_path, cr_file_edit) in cr_file_manifest {
        if cr_file_path.eq(&deleted_file_str) {
            let cr_deleted_files = tokio::fs::read_to_string(
                config.history_path(cr_file_path.as_str(), cr_file_edit.version),
            )
            .await?;
            let mut cr_deleted_list = fpm::cr::resolve_cr_deleted(cr_deleted_files.as_str(), dest)
                .await?
                .into_iter()
                .map(|v| (v.filename.to_string(), v))
                .collect::<std::collections::HashMap<String, fpm::cr::CRDeleted>>();
            let mut already_deleted = vec![];
            for (deleted_file_name, cr_deleted) in cr_deleted_list.iter() {
                let file_edit =
                    if let Some(file_edit) = remote_manifest.get(deleted_file_name.as_str()) {
                        file_edit
                    } else {
                        already_deleted.push(deleted_file_name.to_string());
                        continue;
                    };
                if file_edit.is_deleted() {
                    already_deleted.push(deleted_file_name.clone());
                    continue;
                }
                if !file_edit.version.eq(&cr_deleted.version) {
                    // CloneDeletedRemoteEdited
                    conflicted_file_status.push(fpm::sync_utils::FileStatus::Delete {
                        path: deleted_file_name.to_string(),
                        version: cr_deleted.version,
                        status: fpm::sync_utils::Status::CloneDeletedRemoteEdited(
                            file_edit.version,
                        ),
                    });
                    continue;
                }
            }
            cr_deleted_list = cr_deleted_list
                .into_iter()
                .filter(|(k, _)| !already_deleted.contains(k))
                .collect();
            fpm::cr::create_deleted_files(
                config,
                dest,
                cr_deleted_list.into_values().collect_vec().as_slice(),
            )
            .await?;
            continue;
        }
        let filename = fpm::cr::cr_path_to_file_name(dest, cr_file_path.as_str())?;
        let file_edit = if let Some(file_edit) = remote_manifest.get(filename.as_str()) {
            file_edit
        } else {
            // Added file in CR
            continue;
        };

        let ours_content_bytes =
            tokio::fs::read(config.history_path(cr_file_path.as_str(), cr_file_edit.version))
                .await?;

        // get corresponding track file
        let track_file_edit = if let Some(file_edit) = cr_track_manifest.get(&filename) {
            file_edit
        } else {
            // Added file in CR
            if file_edit.is_deleted() {
                continue;
            }
            let theirs_content_bytes =
                tokio::fs::read(config.history_path(filename.as_str(), file_edit.version)).await?;
            if sha2::Sha256::digest(&ours_content_bytes)
                .eq(&sha2::Sha256::digest(theirs_content_bytes))
            {
                workspace.remove(cr_file_path.as_str());
            } else {
                conflicted_file_status.push(fpm::sync_utils::FileStatus::Add {
                    path: filename.to_string(),
                    content: ours_content_bytes.clone(),
                    status: fpm::sync_utils::Status::CloneAddedRemoteAdded(file_edit.version),
                });
            }
            continue;
        };
        let track_file_path_str =
            config.path_without_root(&config.track_dir().join(&cr_file_path))?;
        let track_file_path =
            config.history_path(track_file_path_str.as_str(), track_file_edit.version);

        let mut tracking_infos = fpm::track::get_tracking_info_(&track_file_path)
            .await?
            .into_iter()
            .map(|v| (v.filename.to_string(), v))
            .collect::<std::collections::HashMap<String, fpm::track::TrackingInfo>>();
        let track_info = if let Some(track_info) = tracking_infos.get_mut(&filename) {
            track_info
        } else {
            // Added file in CR
            if file_edit.is_deleted() {
                continue;
            }
            let theirs_content_bytes =
                tokio::fs::read(config.history_path(filename.as_str(), file_edit.version)).await?;
            if sha2::Sha256::digest(&ours_content_bytes)
                .eq(&sha2::Sha256::digest(theirs_content_bytes))
            {
                workspace.remove(cr_file_path.as_str());
            } else {
                conflicted_file_status.push(fpm::sync_utils::FileStatus::Add {
                    path: filename.to_string(),
                    content: ours_content_bytes.clone(),
                    status: fpm::sync_utils::Status::CloneAddedRemoteAdded(file_edit.version),
                });
            }
            continue;
        };

        if file_edit.is_deleted() {
            if !track_info.version.eq(&file_edit.version) {
                conflicted_file_status.push(fpm::sync_utils::FileStatus::Update {
                    path: filename.to_string(),
                    content: ours_content_bytes.clone(),
                    version: track_info.version,
                    status: fpm::sync_utils::Status::CloneEditedRemoteDeleted(file_edit.version),
                });
            }
            continue;
        }

        if track_info.version.eq(&file_edit.version) {
            // Edited on cr wrt remote's latest version, so ignore
            continue;
        }

        // try to merge
        let ancestor_content = if let Ok(content) =
            tokio::fs::read_to_string(config.history_path(filename.as_str(), track_info.version))
                .await
        {
            content
        } else {
            // binary file like images, can't resolve conflict
            conflicted_file_status.push(fpm::sync_utils::FileStatus::Update {
                path: filename.to_string(),
                content: ours_content_bytes.clone(),
                version: track_info.version,
                status: fpm::sync_utils::Status::Conflict(file_edit.version),
            });
            continue;
        };

        let theirs_content =
            tokio::fs::read_to_string(config.history_path(filename.as_str(), file_edit.version))
                .await?;

        let ours_content = String::from_utf8(ours_content_bytes.clone())?;

        match diffy::MergeOptions::new()
            .set_conflict_style(diffy::ConflictStyle::Merge)
            .merge(&ancestor_content, &ours_content, &theirs_content)
        {
            Ok(data) => {
                fpm::utils::update(config.root.join(&cr_file_path), data.as_bytes()).await?;
                track_info.version = file_edit.version;
                fpm::track::create_tracking_info(
                    config,
                    tracking_infos.into_values().collect_vec().as_slice(),
                    &config.root.join(&cr_file_path),
                )
                .await?;
            }
            Err(_) => {
                // Can't resolve conflict
                conflicted_file_status.push(fpm::sync_utils::FileStatus::Update {
                    path: filename.to_string(),
                    content: ours_content_bytes.clone(),
                    version: track_info.version,
                    status: fpm::sync_utils::Status::Conflict(file_edit.version),
                });
                continue;
            }
        }
    }
    config
        .write_workspace(workspace.into_values().collect_vec().as_slice())
        .await?;
    conflicted_file_status
        .iter()
        .map(|v| fpm::commands::sync_status::print_status(v, false))
        .collect_vec();
    Ok(())
}

async fn merge_cr_into_main(
    config: &fpm::Config,
    src: usize,
    _file: Option<&str>,
) -> fpm::Result<()> {
    let cr_status = config.get_cr_status(src).await?;
    let conflicted_file = cr_status.iter().find(|v| v.is_conflicted());
    if let Some(conflicted_file) = conflicted_file {
        return fpm::usage_error(format!(
            "{} is in conflict state: `{:?}`",
            conflicted_file.get_file_path(),
            conflicted_file.status()
        ));
    }

    let (cr_track_status, cr_file_status) = cr_status.into_iter().fold(
        (vec![], vec![]),
        |(mut cr_track_status, mut cr_file_status), file| {
            if file.get_file_path().starts_with(".tracks/") {
                cr_track_status.push(file);
            } else {
                cr_file_status.push(file);
            }
            (cr_track_status, cr_file_status)
        },
    );

    let cr_track_map = cr_track_status
        .into_iter()
        .map(|v| (v.get_file_path(), v))
        .collect::<std::collections::BTreeMap<String, fpm::sync_utils::FileStatus>>(
    );

    let mut new_file_status: std::collections::BTreeMap<String, fpm::sync_utils::FileStatus> =
        Default::default();
    for file_status in cr_file_status {
        let cr_file_name = file_status.get_file_path();
        let deleted_files_path = config.cr_deleted_file_path(src);
        let deleted_files = config.path_without_root(&deleted_files_path)?;
        if cr_file_name.eq(&deleted_files) {
            // status for deleted files
            let content = if let Some(content) = file_status.get_content() {
                content
            } else {
                continue;
            };

            let deleted_files =
                fpm::cr::resolve_cr_deleted(String::from_utf8(content)?.as_str(), src).await?;
            new_file_status.extend(
                deleted_files
                    .into_iter()
                    .map(|v| (cr_file_name.clone(), v.into_file_status())),
            );
            continue;
        }
        let cr_track_path = format!(".tracks/{}", cr_file_name);
        let file_name = fpm::cr::cr_path_to_file_name(src, cr_file_name.as_str())?;
        let file_content = if let Some(file_content) = file_status.get_content() {
            file_content
        } else {
            continue;
        };

        if let Some(track) = cr_track_map.get(cr_track_path.as_str()) {
            // status for updated files
            let content = track.get_content().ok_or_else(|| fpm::Error::UsageError {
                message: format!("Can't find track content for {}", cr_file_name),
            })?;
            let cr_tracking_infos = fpm::track::resolve_tracking_info(
                String::from_utf8(content)?.as_str(),
                &config.root.join(cr_track_path),
            )
            .await?;
            let tracking_info = cr_tracking_infos
                .into_iter()
                .find(|v| file_name.eq(&v.filename))
                .ok_or_else(|| fpm::Error::UsageError {
                    message: format!("Can't find track info for {}", cr_file_name),
                })?;
            new_file_status.insert(
                cr_file_name,
                fpm::sync_utils::FileStatus::Update {
                    path: file_name,
                    content: file_content,
                    version: tracking_info.version,
                    status: fpm::sync_utils::Status::NoConflict,
                },
            );
        } else {
            // status for updated files
            new_file_status.insert(
                cr_file_name,
                fpm::sync_utils::FileStatus::Add {
                    path: file_name,
                    content: file_content,
                    status: fpm::sync_utils::Status::NoConflict,
                },
            );
        }
    }

    config
        .get_cr_status_wrt_remote_manifest(src, &mut new_file_status)
        .await?;

    let conflicted_file = new_file_status.values().find(|v| v.is_conflicted());
    if let Some(conflicted_file) = conflicted_file {
        return fpm::usage_error(format!(
            "{} is in conflict state: `{:?}`",
            conflicted_file.get_file_path(),
            conflicted_file.status()
        ));
    }

    let changed_files = new_file_status
        .into_values()
        .filter_map(|v| v.sync_request())
        .collect_vec();

    fpm::commands::sync2::sync(config, changed_files).await?;
    Ok(())
}
