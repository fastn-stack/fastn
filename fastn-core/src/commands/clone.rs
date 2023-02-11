pub async fn clone(source: &str) -> fastn_core::Result<()> {
    let clone_response = call_clone_api(source).await?;
    let package_name = clone_response.package_name;
    let current_directory: camino::Utf8PathBuf =
        std::env::current_dir()?.canonicalize()?.try_into()?;
    let root = current_directory.join(&package_name);
    tokio::fs::create_dir_all(&package_name).await?;

    futures::future::join_all(clone_response.files.into_iter().map(|(path, file)| {
        let current_directory = root.clone();
        tokio::spawn(async move {
            fastn_core::utils::update1(&current_directory, path.as_str(), &file).await
        })
    }))
    .await;

    let config = fastn_core::Config::read(Some(root.as_str().to_string()), false, None).await?;
    config.create_clone_workspace().await?;
    config
        .write_clone_available_cr(clone_response.reserved_crs.as_slice())
        .await?;
    Ok(())
}

async fn call_clone_api(
    source: &str,
) -> fastn_core::Result<fastn_core::apis::clone::CloneResponse> {
    #[derive(serde::Deserialize, std::fmt::Debug)]
    struct ApiResponse {
        message: Option<String>,
        data: Option<fastn_core::apis::clone::CloneResponse>,
        success: bool,
    }

    let response: ApiResponse =
        crate::http::get_json(format!("{}/-/clone/", source).as_str()).await?;

    if !response.success {
        return Err(fastn_core::Error::APIResponseError(
            response
                .message
                .unwrap_or_else(|| "Some Error occurred".to_string()),
        ));
    }

    match response.data {
        Some(data) => Ok(data),
        None => Err(fastn_core::Error::APIResponseError(
            "Unexpected API behaviour".to_string(),
        )),
    }
}
