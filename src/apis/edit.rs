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
    req: actix_web::web::Json<EditRequest>,
) -> actix_web::Result<actix_web::HttpResponse> {
    match edit_worker(req.0).await {
        Ok(data) => fpm::apis::success(data),
        Err(err) => fpm::apis::error(
            err.to_string(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

pub(crate) async fn edit_worker(request: EditRequest) -> fpm::Result<EditResponse> {
    let mut config = fpm::Config::read2(None, false).await?;

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

    fpm::utils::update(
        &config.root,
        file_name.as_str(),
        request
            .value
            .unwrap_or_else(|| "".to_string())
            .into_bytes()
            .as_slice(),
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

pub async fn sync() -> actix_web::Result<actix_web::HttpResponse> {
    let config = match fpm::Config::read2(None, false).await {
        Ok(config) => config,
        Err(err) => {
            return fpm::apis::error(
                err.to_string(),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    };
    match fpm::commands::sync::sync(&config, None).await {
        Ok(_) => {
            #[derive(serde::Serialize)]
            struct SyncResponse {
                reload: bool,
            }
            fpm::apis::success(SyncResponse { reload: true })
        }
        Err(err) => fpm::apis::error(
            err.to_string(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
pub struct RevertRequest {
    pub path: String,
}

pub async fn revert(
    req: actix_web::web::Json<RevertRequest>,
) -> actix_web::Result<actix_web::HttpResponse> {
    let config = match fpm::Config::read2(None, false).await {
        Ok(config) => config,
        Err(err) => {
            return fpm::apis::error(
                err.to_string(),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    };

    match fpm::commands::revert::revert(&config, req.0.path.as_str()).await {
        Ok(_) => {
            #[derive(serde::Serialize)]
            struct RevertResponse {
                reload: bool,
            }
            fpm::apis::success(RevertResponse { reload: true })
        }
        Err(err) => fpm::apis::error(
            err.to_string(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}
