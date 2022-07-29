use itertools::Itertools;

pub async fn sync_status(config: &fpm::Config, source: Option<&str>) -> fpm::Result<()> {
    let get_files_status = fpm::sync_utils::get_files_status(config).await?;
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

fn print_status(file_status: &fpm::sync_utils::FileStatus, print_untracked: bool) {
    let (file_status, path, status) = match file_status {
        fpm::sync_utils::FileStatus::Add { path, status, .. } => ("Added", path, status),
        fpm::sync_utils::FileStatus::Update { path, status, .. } => ("Updated", path, status),
        fpm::sync_utils::FileStatus::Delete { path, status, .. } => ("Deleted", path, status),
        fpm::sync_utils::FileStatus::Untracked { path, .. } => {
            if print_untracked {
                println!("Untracked: {}", path);
            }
            return;
        }
    };
    match status {
        fpm::sync_utils::Status::Conflict(_) => println!("Conflicted: {}", path),
        fpm::sync_utils::Status::ClientEditedServerDeleted(_) => {
            println!("ClientEditedServerDeleted: {}", path)
        }
        fpm::sync_utils::Status::ClientDeletedServerEdited(_) => {
            println!("ClientDeletedServerEdited: {}", path)
        }
        fpm::sync_utils::Status::NoConflict => println!("{}: {}", file_status, path),
    }
}
