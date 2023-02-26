pub async fn merge(
    config: &fastn_core::Config,
    src: Option<&str>,
    dest: &str,
    file: Option<&str>,
) -> fastn_core::Result<()> {
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
    _config: &fastn_core::Config,
    _src: usize,
    _dest: usize,
    _file: Option<&str>,
) -> fastn_core::Result<()> {
    unimplemented!()
}

async fn merge_main_into_cr(
    config: &fastn_core::Config,
    dest: usize,
    file: Option<&str>,
) -> fastn_core::Result<()> {
    use itertools::Itertools;
    use sha2::Digest;

    let remote_manifest: std::collections::BTreeMap<String, fastn_core::history::FileEdit> = config
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
    let mut new_file_status: std::collections::BTreeMap<
        String,
        fastn_core::sync_utils::FileStatus,
    > = Default::default();

    // True: if file: Option<&str> has some value and it's processed
    let mut file_processed = false;

    for (cr_file_path, cr_file_edit) in cr_file_manifest.iter() {
        if file_processed {
            break;
        }
        if cr_file_path.eq(&deleted_file_str) {
            let cr_deleted_files = tokio::fs::read_to_string(
                config.history_path(cr_file_path.as_str(), cr_file_edit.version),
            )
            .await?;
            let mut cr_deleted_list =
                fastn_core::cr::resolve_cr_deleted(cr_deleted_files.as_str(), dest)
                    .await?
                    .into_iter()
                    .map(|v| (v.filename.to_string(), v))
                    .collect::<std::collections::HashMap<String, fastn_core::cr::CRDeleted>>();
            let mut already_deleted = vec![];
            for (deleted_file_name, cr_deleted) in cr_deleted_list.iter() {
                if file_processed {
                    break;
                }
                if let Some(file) = file {
                    if deleted_file_name.ne(file) {
                        continue;
                    } else {
                        file_processed = true;
                    }
                }
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
                    conflicted_file_status.push(fastn_core::sync_utils::FileStatus::Delete {
                        path: deleted_file_name.to_string(),
                        version: cr_deleted.version,
                        status: fastn_core::sync_utils::Status::CloneDeletedRemoteEdited(
                            file_edit.version,
                        ),
                    });
                    continue;
                }
            }
            if !already_deleted.is_empty() {
                cr_deleted_list.retain(|k, _| !already_deleted.contains(k));
                new_file_status.insert(
                    cr_file_path.to_string(),
                    fastn_core::sync_utils::FileStatus::Update {
                        path: cr_file_path.to_string(),
                        content: fastn_core::cr::generate_deleted_files_content(
                            cr_deleted_list.into_values().collect_vec().as_slice(),
                        )
                        .into_bytes(),
                        version: cr_file_edit.version,
                        status: fastn_core::sync_utils::Status::NoConflict,
                    },
                );
            }
            continue;
        }
        let filename = fastn_core::cr::cr_path_to_file_name(dest, cr_file_path.as_str())?;
        if let Some(file) = file {
            if filename.ne(file) {
                continue;
            } else {
                file_processed = true;
            }
        }
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
        let track_file_path_str =
            config.path_without_root(&config.track_dir().join(cr_file_path))?;
        let track_file_edit = match cr_track_manifest.get(&track_file_path_str) {
            Some(file_edit) => file_edit,
            _ => {
                // Added file in CR
                if file_edit.is_deleted() {
                    continue;
                }
                let theirs_content_bytes =
                    tokio::fs::read(config.history_path(filename.as_str(), file_edit.version))
                        .await?;
                if sha2::Sha256::digest(&ours_content_bytes)
                    .eq(&sha2::Sha256::digest(theirs_content_bytes))
                {
                    new_file_status.insert(
                        cr_file_path.to_string(),
                        fastn_core::sync_utils::FileStatus::Delete {
                            path: cr_file_path.to_string(),
                            version: cr_file_edit.version,
                            status: fastn_core::sync_utils::Status::NoConflict,
                        },
                    );
                } else {
                    conflicted_file_status.push(fastn_core::sync_utils::FileStatus::Add {
                        path: filename.to_string(),
                        content: ours_content_bytes.clone(),
                        status: fastn_core::sync_utils::Status::CloneAddedRemoteAdded(
                            file_edit.version,
                        ),
                    });
                }
                continue;
            }
        };

        let track_file_path =
            config.history_path(track_file_path_str.as_str(), track_file_edit.version);

        let mut tracking_infos = fastn_core::track::get_tracking_info_(&track_file_path)
            .await?
            .into_iter()
            .map(|v| (v.filename.to_string(), v))
            .collect::<std::collections::HashMap<String, fastn_core::track::TrackingInfo>>();
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
                new_file_status.insert(
                    cr_file_path.to_string(),
                    fastn_core::sync_utils::FileStatus::Delete {
                        path: cr_file_path.to_string(),
                        version: cr_file_edit.version,
                        status: fastn_core::sync_utils::Status::NoConflict,
                    },
                );
            } else {
                conflicted_file_status.push(fastn_core::sync_utils::FileStatus::Add {
                    path: filename.to_string(),
                    content: ours_content_bytes.clone(),
                    status: fastn_core::sync_utils::Status::CloneAddedRemoteAdded(
                        file_edit.version,
                    ),
                });
            }
            continue;
        };

        if file_edit.is_deleted() {
            if !track_info.version.eq(&file_edit.version) {
                conflicted_file_status.push(fastn_core::sync_utils::FileStatus::Update {
                    path: filename.to_string(),
                    content: ours_content_bytes.clone(),
                    version: track_info.version,
                    status: fastn_core::sync_utils::Status::CloneEditedRemoteDeleted(
                        file_edit.version,
                    ),
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
            conflicted_file_status.push(fastn_core::sync_utils::FileStatus::Update {
                path: filename.to_string(),
                content: ours_content_bytes.clone(),
                version: track_info.version,
                status: fastn_core::sync_utils::Status::Conflict(file_edit.version),
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
                new_file_status.insert(
                    cr_file_path.to_string(),
                    fastn_core::sync_utils::FileStatus::Update {
                        path: cr_file_path.to_string(),
                        content: data.into_bytes(),
                        version: cr_file_edit.version,
                        status: fastn_core::sync_utils::Status::NoConflict,
                    },
                );
                track_info.version = file_edit.version;
                new_file_status.insert(
                    track_file_path_str.to_string(),
                    fastn_core::sync_utils::FileStatus::Update {
                        path: track_file_path_str.to_string(),
                        content: fastn_core::track::generate_tracking_info_content(
                            tracking_infos.into_values().collect_vec().as_slice(),
                        )
                        .into_bytes(),
                        version: cr_file_edit.version,
                        status: fastn_core::sync_utils::Status::NoConflict,
                    },
                );
            }
            Err(_) => {
                // Can't resolve conflict
                conflicted_file_status.push(fastn_core::sync_utils::FileStatus::Update {
                    path: filename.to_string(),
                    content: ours_content_bytes.clone(),
                    version: track_info.version,
                    status: fastn_core::sync_utils::Status::Conflict(file_edit.version),
                });
                continue;
            }
        }
    }
    conflicted_file_status
        .iter()
        .map(|v| fastn_core::commands::sync_status::print_status(v, false))
        .collect_vec();

    if conflicted_file_status.is_empty() {
        let changed_files = new_file_status
            .into_values()
            .filter_map(|v| v.sync_request(None))
            .collect_vec();
        fastn_core::apis::sync2::do_sync(config, changed_files.as_slice()).await?;
    }
    Ok(())
}

async fn merge_cr_into_main(
    config: &fastn_core::Config,
    src: usize,
    file: Option<&str>,
) -> fastn_core::Result<()> {
    use itertools::Itertools;
    use sha2::Digest;

    //TODO: check if cr is closed
    let remote_manifest: std::collections::BTreeMap<String, fastn_core::history::FileEdit> = config
        .get_remote_manifest(true)
        .await?
        .into_iter()
        .filter(|(k, _)| !(k.starts_with("-/") || k.starts_with(".tracks/")))
        .collect();
    let cr_manifest = config.get_cr_manifest(src).await?;
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
    let cr_statuses = {
        let mut cr_statuses = cr_file_manifest
            .iter()
            .filter_map(|(k, v)| {
                if v.is_deleted() {
                    None
                } else if let Some(file) = file {
                    let cr_file_name = format!("{}/{}", fastn_core::cr::cr_path(src), file);
                    if cr_file_name.eq(k) {
                        Some(fastn_core::sync_utils::FileStatus::Delete {
                            path: k.to_string(),
                            version: v.version,
                            status: fastn_core::sync_utils::Status::NoConflict,
                        })
                    } else {
                        None
                    }
                } else {
                    Some(fastn_core::sync_utils::FileStatus::Delete {
                        path: k.to_string(),
                        version: v.version,
                        status: fastn_core::sync_utils::Status::NoConflict,
                    })
                }
            })
            .collect_vec();

        let cr_track_statuses = cr_track_manifest
            .iter()
            .filter_map(|(k, v)| {
                if v.is_deleted() {
                    None
                } else if let Some(file) = file {
                    let cr_track_file_name = config
                        .path_without_root(&config.track_path(&config.cr_path(src).join(file)))
                        .unwrap(); // This is safe
                    if cr_track_file_name.eq(k) {
                        Some(fastn_core::sync_utils::FileStatus::Delete {
                            path: k.to_string(),
                            version: v.version,
                            status: fastn_core::sync_utils::Status::NoConflict,
                        })
                    } else {
                        None
                    }
                } else {
                    Some(fastn_core::sync_utils::FileStatus::Delete {
                        path: k.to_string(),
                        version: v.version,
                        status: fastn_core::sync_utils::Status::NoConflict,
                    })
                }
            })
            .collect_vec();
        cr_statuses.extend(cr_track_statuses);
        cr_statuses
    };

    let mut new_file_status: std::collections::BTreeMap<
        String,
        fastn_core::sync_utils::FileStatus,
    > = Default::default();
    let mut conflicted_file_status = vec![];
    let deleted_files_path = config.cr_deleted_file_path(src);
    let deleted_files = config.path_without_root(&deleted_files_path)?;
    let mut file_processed = false;
    for (cr_file_name, cr_file_edit) in cr_file_manifest.iter() {
        if file_processed {
            break;
        }
        if cr_file_name.eq(&deleted_files) {
            // status for deleted files
            let cr_deleted_files = tokio::fs::read_to_string(
                config.history_path(cr_file_name.as_str(), cr_file_edit.version),
            )
            .await?;
            let cr_deleted_list =
                fastn_core::cr::resolve_cr_deleted(cr_deleted_files.as_str(), src).await?;

            for cr_delete in cr_deleted_list {
                if file_processed {
                    break;
                }
                if let Some(file) = file {
                    if cr_delete.filename.ne(file) {
                        continue;
                    } else {
                        file_processed = true;
                    }
                }
                let file_edit =
                    if let Some(file_edit) = remote_manifest.get(cr_delete.filename.as_str()) {
                        file_edit
                    } else {
                        return fastn_core::usage_error(format!(
                            "Can't find history of `{}` which is marked to be deleted in CR#{}",
                            cr_delete.filename, src
                        ));
                    };
                if file_edit.is_deleted() {
                    continue;
                }
                if !file_edit.version.eq(&cr_delete.version) {
                    conflicted_file_status.push(fastn_core::sync_utils::FileStatus::Delete {
                        path: cr_delete.filename.to_string(),
                        version: cr_delete.version,
                        status: fastn_core::sync_utils::Status::CloneDeletedRemoteEdited(
                            file_edit.version,
                        ),
                    });
                    continue;
                }
                new_file_status
                    .insert(cr_delete.filename.to_string(), cr_delete.into_file_status());
            }
            continue;
        }

        let cr_file_path = config.history_path(cr_file_name.as_str(), cr_file_edit.version);
        let filename = fastn_core::cr::cr_path_to_file_name(src, cr_file_path.as_str())?;
        if let Some(file) = file {
            if filename.ne(file) {
                continue;
            } else {
                file_processed = true;
            }
        }

        let cr_file_content = tokio::fs::read(&cr_file_path).await?;
        let file_edit = if let Some(file_edit) = remote_manifest.get(filename.as_str()) {
            file_edit
        } else {
            // Added file in CR
            new_file_status.insert(
                filename.to_string(),
                fastn_core::sync_utils::FileStatus::Add {
                    path: cr_file_name.to_string(),
                    content: cr_file_content.clone(),
                    status: fastn_core::sync_utils::Status::NoConflict,
                },
            );
            continue;
        };

        let track_file_path_str =
            config.path_without_root(&config.track_dir().join(cr_file_name))?;
        let track_file_edit = match cr_track_manifest.get(&track_file_path_str) {
            Some(file_edit) if !file_edit.is_deleted() => file_edit,
            _ => {
                // Added file in CR
                new_file_status.insert(
                    filename.to_string(),
                    fastn_core::sync_utils::FileStatus::Add {
                        path: cr_file_name.to_string(),
                        content: cr_file_content.clone(),
                        status: fastn_core::sync_utils::Status::NoConflict,
                    },
                );
                continue;
            }
        };

        let ours_content_bytes =
            tokio::fs::read(config.history_path(cr_file_path.as_str(), cr_file_edit.version))
                .await?;

        let track_file_path =
            config.history_path(track_file_path_str.as_str(), track_file_edit.version);

        let mut tracking_infos = fastn_core::track::get_tracking_info_(&track_file_path)
            .await?
            .into_iter()
            .map(|v| (v.filename.to_string(), v))
            .collect::<std::collections::HashMap<String, fastn_core::track::TrackingInfo>>();
        let track_info = if let Some(track_info) = tracking_infos.get_mut(&filename) {
            track_info
        } else {
            // Added file in CR
            if file_edit.is_deleted() {
                new_file_status.insert(
                    filename.to_string(),
                    fastn_core::sync_utils::FileStatus::Add {
                        path: cr_file_name.to_string(),
                        content: cr_file_content.clone(),
                        status: fastn_core::sync_utils::Status::NoConflict,
                    },
                );
            }
            let theirs_content_bytes =
                tokio::fs::read(config.history_path(filename.as_str(), file_edit.version)).await?;
            if !sha2::Sha256::digest(&ours_content_bytes)
                .eq(&sha2::Sha256::digest(theirs_content_bytes))
            {
                conflicted_file_status.push(fastn_core::sync_utils::FileStatus::Add {
                    path: filename.to_string(),
                    content: ours_content_bytes.clone(),
                    status: fastn_core::sync_utils::Status::CloneAddedRemoteAdded(
                        file_edit.version,
                    ),
                });
            }
            continue;
        };

        if file_edit.is_deleted() {
            if !track_info.version.eq(&file_edit.version) {
                conflicted_file_status.push(fastn_core::sync_utils::FileStatus::Update {
                    path: filename.to_string(),
                    content: ours_content_bytes.clone(),
                    version: track_info.version,
                    status: fastn_core::sync_utils::Status::CloneEditedRemoteDeleted(
                        file_edit.version,
                    ),
                });
            }
            continue;
        }

        if track_info.version.eq(&file_edit.version) {
            // Edited on cr wrt remote's latest version
            new_file_status.insert(
                filename.to_string(),
                fastn_core::sync_utils::FileStatus::Update {
                    path: filename,
                    content: cr_file_content,
                    version: track_info.version,
                    status: fastn_core::sync_utils::Status::NoConflict,
                },
            );
            continue;
        }

        let ancestor_content = if let Ok(content) =
            tokio::fs::read_to_string(config.history_path(filename.as_str(), track_info.version))
                .await
        {
            content
        } else {
            // binary file like images, can't resolve conflict
            conflicted_file_status.push(fastn_core::sync_utils::FileStatus::Update {
                path: filename.to_string(),
                content: ours_content_bytes.clone(),
                version: track_info.version,
                status: fastn_core::sync_utils::Status::Conflict(file_edit.version),
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
                new_file_status.insert(
                    filename.to_string(),
                    fastn_core::sync_utils::FileStatus::Update {
                        path: filename.to_string(),
                        content: data.into_bytes(),
                        version: cr_file_edit.version,
                        status: fastn_core::sync_utils::Status::NoConflict,
                    },
                );
            }
            Err(_) => {
                // Can't resolve conflict
                conflicted_file_status.push(fastn_core::sync_utils::FileStatus::Update {
                    path: filename.to_string(),
                    content: ours_content_bytes.clone(),
                    version: track_info.version,
                    status: fastn_core::sync_utils::Status::Conflict(file_edit.version),
                });
                continue;
            }
        }
    }

    conflicted_file_status
        .iter()
        .map(|v| fastn_core::commands::sync_status::print_status(v, false))
        .collect_vec();

    if conflicted_file_status.is_empty() {
        let mut sync_request_files = new_file_status
            .into_values()
            .filter_map(|v| v.sync_request(Some(src)))
            .collect_vec();
        sync_request_files.extend(cr_statuses.into_iter().filter_map(|v| v.sync_request(None)));
        if file.is_none() {
            let about_status = add_close_cr_status(config, src, &cr_file_manifest).await?;
            if let Some(sync_req) = about_status.sync_request(None) {
                sync_request_files.push(sync_req);
            }
        }
        fastn_core::apis::sync2::do_sync(config, sync_request_files.as_slice()).await?;
    }

    Ok(())
}

async fn add_close_cr_status(
    config: &fastn_core::Config,
    cr: usize,
    cr_file_manifest: &std::collections::HashMap<String, fastn_core::history::FileEdit>,
) -> fastn_core::Result<fastn_core::sync_utils::FileStatus> {
    let cr_about_path_str = config.path_without_root(&config.cr_meta_path(cr))?;
    let cr_about_file_edit = cr_file_manifest
        .get(cr_about_path_str.as_str())
        .ok_or_else(|| fastn_core::Error::CRAboutNotFound {
            message: "Missing cr about".to_string(),
            cr_number: cr,
        })?;
    if cr_about_file_edit.is_deleted() {
        return Err(fastn_core::Error::CRAboutNotFound {
            message: "Missing cr about".to_string(),
            cr_number: cr,
        });
    }
    let cr_about_path = config.history_path(cr_about_path_str.as_str(), cr_about_file_edit.version);
    let cr_meta_content = tokio::fs::read_to_string(cr_about_path).await?;
    let mut cr_about = fastn_core::cr::resolve_cr_meta(cr_meta_content.as_str(), cr).await?;
    cr_about.open = false;
    let cr_close_content = fastn_core::cr::generate_cr_meta_content(&cr_about);
    Ok(fastn_core::sync_utils::FileStatus::Update {
        path: cr_about_path_str,
        content: cr_close_content.into_bytes(),
        version: cr_about_file_edit.version,
        status: fastn_core::sync_utils::Status::NoConflict,
    })
}

/*async fn merge_cr_into_main(
    config: &fastn_core::Config,
    src: usize,
    _file: Option<&str>,
) -> fastn_core::Result<()> {
    let cr_status = config.get_cr_status(src).await?;
    let conflicted_file = cr_status.iter().find(|v| v.is_conflicted());
    if let Some(conflicted_file) = conflicted_file {
        return fastn_core::usage_error(format!(
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
        .collect::<std::collections::BTreeMap<String, fastn_core::sync_utils::FileStatus>>(
    );

    let mut new_file_status: std::collections::BTreeMap<String, fastn_core::sync_utils::FileStatus> =
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
                fastn_core::cr::resolve_cr_deleted(String::from_utf8(content)?.as_str(), src).await?;
            new_file_status.extend(
                deleted_files
                    .into_iter()
                    .map(|v| (cr_file_name.clone(), v.into_file_status())),
            );
            continue;
        }
        let cr_track_path = format!(".tracks/{}", cr_file_name);
        let file_name = fastn_core::cr::cr_path_to_file_name(src, cr_file_name.as_str())?;
        let file_content = if let Some(file_content) = file_status.get_content() {
            file_content
        } else {
            continue;
        };

        if let Some(track) = cr_track_map.get(cr_track_path.as_str()) {
            // status for updated files
            let content = track.get_content().ok_or_else(|| fastn_core::Error::UsageError {
                message: format!("Can't find track content for {}", cr_file_name),
            })?;
            let cr_tracking_infos = fastn_core::track::resolve_tracking_info(
                String::from_utf8(content)?.as_str(),
                &config.root.join(cr_track_path),
            )
            .await?;
            let tracking_info = cr_tracking_infos
                .into_iter()
                .find(|v| file_name.eq(&v.filename))
                .ok_or_else(|| fastn_core::Error::UsageError {
                    message: format!("Can't find track info for {}", cr_file_name),
                })?;
            new_file_status.insert(
                cr_file_name,
                fastn_core::sync_utils::FileStatus::Update {
                    path: file_name,
                    content: file_content,
                    version: tracking_info.version,
                    status: fastn_core::sync_utils::Status::NoConflict,
                },
            );
        } else {
            // status for updated files
            new_file_status.insert(
                cr_file_name,
                fastn_core::sync_utils::FileStatus::Add {
                    path: file_name,
                    content: file_content,
                    status: fastn_core::sync_utils::Status::NoConflict,
                },
            );
        }
    }

    config
        .get_cr_status_wrt_remote_manifest(src, &mut new_file_status)
        .await?;

    let conflicted_file = new_file_status.values().find(|v| v.is_conflicted());
    if let Some(conflicted_file) = conflicted_file {
        return fastn_core::usage_error(format!(
            "{} is in conflict state: `{:?}`",
            conflicted_file.get_file_path(),
            conflicted_file.status()
        ));
    }

    let changed_files = new_file_status
        .into_values()
        .filter_map(|v| v.sync_request())
        .collect_vec();

    fastn_core::commands::sync2::sync(config, changed_files).await?;
    Ok(())
}*/
