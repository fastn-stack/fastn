use itertools::Itertools;

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
    _config: &fpm::Config,
    _src: usize,
    _file: Option<&str>,
) -> fpm::Result<()> {
    unimplemented!()
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
