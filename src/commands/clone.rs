use std::convert::TryInto;

pub async fn clone(source: &str) -> fpm::Result<()> {
    let clone_response = call_clone_api(source)?;
    let package_name = clone_response.package_name;
    let current_directory: camino::Utf8PathBuf =
        std::env::current_dir()?.canonicalize()?.try_into()?;
    let root = current_directory.join(&package_name);
    tokio::fs::create_dir_all(&package_name).await?;

    futures::future::join_all(clone_response.files.into_iter().map(|(path, file)| {
        let current_directory = root.clone();
        tokio::spawn(
            async move { fpm::utils::update1(&current_directory, path.as_str(), &file).await },
        )
    }))
    .await;

    let config = fpm::Config::read(Some(root.as_str().to_string()), false).await?;
    config.create_clone_workspace().await?;
    config
        .write_clone_available_cr(clone_response.reserved_crs.as_slice())
        .await?;
    Ok(())
}

fn call_clone_api(source: &str) -> fpm::Result<fpm::apis::clone::CloneResponse> {
    #[derive(serde::Deserialize, std::fmt::Debug)]
    struct ApiResponse {
        message: Option<String>,
        data: Option<fpm::apis::clone::CloneResponse>,
        success: bool,
    }

    let source_url = format!("{}/-/clone/", source);
    let mut response = reqwest::Client::new()
        .get(source_url.as_str())
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .send()?;
    let text = response.text()?;
    let response: ApiResponse = serde_json::from_str(text.as_str())?;

    if !response.success {
        return Err(fpm::Error::APIResponseError(
            response
                .message
                .unwrap_or_else(|| "Some Error occurred".to_string()),
        ));
    }

    match response.data {
        Some(data) => Ok(data),
        None => Err(fpm::Error::APIResponseError(
            "Unexpected API behaviour".to_string(),
        )),
    }
}
