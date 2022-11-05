#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
pub struct EditRequest {
    pub url: String,
    pub value: Option<String>,
    pub path: String,
    pub operation: Option<String>, // todo: convert it to enum
    pub data: Option<String>,
}

impl EditRequest {
    pub(crate) fn is_delete(&self) -> bool {
        matches!(self.operation.as_ref(), Some(v) if v.eq("delete"))
    }

    pub(crate) fn is_rename(&self) -> bool {
        matches!(self.operation.as_ref(), Some(v) if v.eq("rename"))
    }
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
pub struct EditResponse {
    pub path: String,
    pub url: Option<String>,
    pub reload: bool,
}

pub async fn edit(
    req: &fpm::http::Request,
    req_data: EditRequest,
) -> fpm::Result<fpm::http::Response> {
    let mut config = match fpm::Config::read(None, false, Some(req)).await {
        Ok(config) => config,
        Err(err) => return fpm::http::api_error(err.to_string()),
    };
    config.current_document = Some(req_data.path.to_string());

    match config.can_write(req, req_data.path.as_str()).await {
        Ok(can_write) => {
            if !can_write {
                return Ok(fpm::unauthorised!(
                    "You are unauthorized to access: {}",
                    req_data.path.as_str()
                ));
            }
        }
        Err(e) => {
            return Ok(fpm::server_error!(
                "FPM-Error: can_read error: {}, {:?}",
                req_data.path.as_str(),
                e
            ));
        }
    };

    match edit_worker(config, req_data).await {
        Ok(data) => fpm::http::api_ok(data),
        Err(err) => fpm::http::api_error(err.to_string()),
    }
}

pub(crate) async fn edit_worker(
    config: fpm::Config,
    request: EditRequest,
) -> fpm::Result<EditResponse> {
    if request.is_delete() {
        let path = config.root.join(&request.path);
        if path.is_dir() {
            tokio::fs::remove_dir_all(&path).await?;
        } else if path.is_file() {
            tokio::fs::remove_file(&path).await?;
        }
        return Ok(EditResponse {
            path: request.path,
            url: None,
            reload: true,
        });
    }

    if request.is_rename() {
        let rename = match request.data {
            Some(v) if !v.is_empty() => v,
            _ => {
                return Err(fpm::Error::APIResponseError(
                    "rename value should present".to_string(),
                ));
            }
        };

        let new_path = if let Some((p, _)) = request.path.trim_end_matches('/').rsplit_once('/') {
            format!("{}/{}", p, rename)
        } else {
            rename
        };

        tokio::fs::rename(config.root.join(&request.path), config.root.join(new_path)).await?;

        // TODO: redirect to renamed file, if folder so it will redirect to renamed folder with
        // index.ftd, if index.ftd does not exists so it will redirected to main project index.ftd

        return Ok(EditResponse {
            path: request.path,
            url: None,
            reload: true,
        });
    }

    // Handle Modify and Add
    let (file_name, url, before_update_status) = if let Ok(path) = config
        .get_file_path_and_resolve(request.path.as_str())
        .await
    {
        let snapshots = fpm::snapshot::get_latest_snapshots(&config.root).await?;
        let workspaces = fpm::snapshot::get_workspace(&config).await?;

        let file = fpm::get_file(
            config.package.name.to_string(),
            &config.root.join(&path),
            &config.root,
        )
        .await?;
        let before_update_status =
            fpm::commands::status::get_file_status(&file, &snapshots, &workspaces).await?;

        (path.to_string(), None, Some(before_update_status))
    } else if request.path.ends_with('/') {
        let path = format!("{}index.ftd", request.path);
        (
            path.to_string(),
            Some(format!("-/view-src/{}", path.trim_start_matches('/'))),
            None,
        )
    } else {
        (
            request.path.to_string(),
            Some(format!(
                "-/view-src/{}",
                request.path.trim_start_matches('/')
            )),
            None,
        )
    };

    fpm::utils::update1(
        &config.root,
        file_name.as_str(),
        request.value.unwrap_or_default().into_bytes().as_slice(),
    )
    .await?;

    if let Some(before_update_status) = before_update_status {
        let snapshots = fpm::snapshot::get_latest_snapshots(&config.root).await?;
        let workspaces = fpm::snapshot::get_workspace(&config).await?;
        let file = fpm::get_file(
            config.package.name.to_string(),
            &config.root.join(&file_name),
            &config.root,
        )
        .await?;
        let after_update_status =
            fpm::commands::status::get_file_status(&file, &snapshots, &workspaces).await?;
        if !before_update_status.eq(&after_update_status) {
            return Ok(EditResponse {
                path: request.path,
                url: Some(format!("-/view-src/{}", file_name.trim_start_matches('/'))),
                reload: false,
            });
        }
    }

    Ok(EditResponse {
        path: request.path,
        url,
        reload: false,
    })
}

pub async fn sync(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let config = match fpm::Config::read(None, false, Some(&req)).await {
        Ok(config) => config,
        Err(err) => return fpm::http::api_error(err.to_string()),
    };
    match fpm::commands::sync::sync(&config, None).await {
        Ok(_) => {
            #[derive(serde::Serialize)]
            struct SyncResponse {
                reload: bool,
            }
            fpm::http::api_ok(SyncResponse { reload: true })
        }
        Err(err) => fpm::http::api_error(err.to_string()),
    }
}

#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
pub struct RevertRequest {
    pub path: String,
}

pub async fn revert(
    req: &fpm::http::Request,
    rev: RevertRequest,
) -> fpm::Result<fpm::http::Response> {
    let config = match fpm::Config::read(None, false, Some(req)).await {
        Ok(config) => config,
        Err(err) => return fpm::http::api_error(err.to_string()),
    };

    match fpm::commands::revert::revert(&config, rev.path.as_str()).await {
        Ok(_) => {
            #[derive(serde::Serialize)]
            struct RevertResponse {
                reload: bool,
            }
            fpm::http::api_ok(RevertResponse { reload: true })
        }
        Err(err) => fpm::http::api_error(err.to_string()),
    }
}
