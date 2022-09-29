pub async fn sync_status(config: &fpm::Config, source: Option<&str>) -> fpm::Result<()> {
    use itertools::Itertools;

    let get_files_status = config.get_files_status().await?;
    if let Some(source) = source {
        if let Some(file_status) = get_files_status
            .iter()
            .find(|v| v.get_file_path().eq(source))
        {
            print_status(file_status, true);
            return Ok(());
        }
        return Err(fpm::Error::UsageError {
            message: format!("{} not found", source),
        });
    }
    get_files_status
        .iter()
        .map(|v| print_status(v, false))
        .collect_vec();
    Ok(())
}

pub(crate) fn print_status(file_status: &fpm::sync_utils::FileStatus, print_untracked: bool) {
    let (file_status, path, status) = match file_status {
        fpm::sync_utils::FileStatus::Add { path, status, .. } => ("Added", path, status),
        fpm::sync_utils::FileStatus::Update { path, status, .. } => ("Updated", path, status),
        fpm::sync_utils::FileStatus::Delete { path, status, .. } => ("Deleted", path, status),
        fpm::sync_utils::FileStatus::Uptodate { path, .. } => {
            if print_untracked {
                println!("Up-to-date: {}", path);
            }
            return;
        }
    };
    match status {
        fpm::sync_utils::Status::Conflict(_) => println!("Conflicted: {}", path),
        fpm::sync_utils::Status::CloneEditedRemoteDeleted(_) => {
            println!("CloneEditedRemoteDeleted: {}", path)
        }
        fpm::sync_utils::Status::CloneDeletedRemoteEdited(_) => {
            println!("CloneDeletedRemoteEdited: {}", path)
        }
        fpm::sync_utils::Status::NoConflict => println!("{}: {}", file_status, path),
        fpm::sync_utils::Status::CloneAddedRemoteAdded(_) => {
            println!("CloneAddedRemoteAdded: {}", path)
        }
    }
}
