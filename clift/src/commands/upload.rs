pub async fn upload_file(
    site_slug: &str,
    file: &str,
) -> Result<(), crate::commands::upload::UploadError> {
    let current_dir = std::env::current_dir().map_err(|_| UploadError::CanNotReadCurrentDir)?;
    let file = clift::utils::path_to_content(&current_dir, &current_dir.join(file)).await?;
    initiate_upload(
        &current_dir,
        clift::api::InitiateUploadRequest::File {
            site: site_slug.to_string(),
            file,
        },
    )
    .await
}

pub async fn upload_folder(
    site_slug: &str,
    folder: &str,
) -> Result<(), crate::commands::upload::UploadError> {
    let current_dir = std::env::current_dir().map_err(|_| UploadError::CanNotReadCurrentDir)?;
    let files = clift::utils::get_local_files(&current_dir, folder).await?;
    initiate_upload(
        &current_dir,
        clift::api::InitiateUploadRequest::Folder {
            site: site_slug.to_string(),
            files,
            folder: folder.to_string(),
        },
    )
    .await
}

pub async fn initiate_upload(
    current_dir: &std::path::Path,
    to_upload: clift::api::InitiateUploadRequest,
) -> Result<(), UploadError> {
    let update_token = clift::utils::update_token()?;
    println!("Initialing Upload....");

    let site_slug = to_upload.get_site();
    let data = clift::api::initiate_upload(to_upload, &update_token).await?;

    if let (Some(pre_signed_request), Some(tejar_file_id)) =
        (data.pre_signed_request.clone(), data.tejar_file_id)
    {
        upload_(&data, pre_signed_request, tejar_file_id, current_dir).await?;
    } else {
        println!("Nothing to upload!");
    }

    println!("Committing Upload...");

    clift::api::commit_upload(site_slug.as_str(), &data, &update_token).await?;

    println!("Upload Done");
    Ok(())
}

async fn upload_(
    data: &clift::api::InitiateUploadResponse,
    pre_signed_request: clift::api::PreSignedRequest,
    tejar_file_id: i64,
    current_dir: &std::path::Path,
) -> Result<(), UploadError> {
    let mut uploader = match std::env::var("DEBUG_USE_TEJAR_FOLDER") {
        Ok(path) => {
            let path = std::path::PathBuf::from(path).join(format!("{}.tejar", tejar_file_id));
            println!("DEBUG_USE_TEJAR_FOLDER: {path:?}");
            clift::utils::Uploader::debug(&path).await?
        }
        Err(_) => {
            println!("using s3");
            clift::utils::Uploader::s3(pre_signed_request)
        }
    };

    upload_files(
        &mut uploader,
        data.new_files.as_slice(),
        current_dir,
        "Added",
    )
    .await?;
    upload_files(
        &mut uploader,
        data.updated_files.as_slice(),
        current_dir,
        "Updated",
    )
    .await?;

    Ok(uploader.commit().await?)
}

async fn upload_files(
    uploader: &mut clift::utils::Uploader,
    files: &[String],
    current_dir: &std::path::Path,
    status: &str,
) -> Result<(), UploadError> {
    for file_name in files.iter() {
        uploader.upload(&current_dir.join(file_name)).await?;
        println!("{file_name}.... {status}");
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum UploadError {
    #[error("CanNotReadCurrentDir")]
    CanNotReadCurrentDir,

    #[error("Cant Read Tokens: {0}")]
    CantReadTokens(#[from] clift::utils::UpdateTokenError),

    #[error("CantInitiateUpload: {0}")]
    CantInitiateUpload(#[from] clift::api::InitiateUploadError),

    #[error("CantCommitUpload: {0}")]
    CantCommitUpload(#[from] clift::api::CommitUploadError),

    #[error("CantUpload: {0}")]
    CantUpload(#[from] clift::utils::UploaderError),

    #[error("cant get local files: {0}")]
    CantGetLocalFiles(#[from] clift::utils::GetLocalFilesError),
}
