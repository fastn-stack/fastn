use itertools::Itertools;

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
pub struct CloneResponse {
    pub package_name: String,
    pub files: std::collections::BTreeMap<String, Vec<u8>>,
    pub reserved_crs: Vec<i32>,
}

pub async fn clone() -> actix_web::Result<actix_web::HttpResponse> {
    match clone_worker().await {
        Ok(data) => fpm::apis::success(data),
        Err(err) => fpm::apis::error(
            err.to_string(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

async fn clone_worker() -> fpm::Result<CloneResponse> {
    let config = fpm::Config::read(None, false).await?;
    let all_files = config
        .get_all_file_path(&config.package, Default::default())?
        .into_iter()
        .filter(|v| !config.server_cr().eq(v))
        .collect_vec();
    let root = config.get_root_for_package(&config.package);
    let files = futures::future::join_all(
        all_files
            .into_iter()
            .map(|x| {
                let root = root.clone();
                tokio::spawn(async move {
                    tokio::fs::read(&x)
                        .await
                        .map_err(fpm::Error::IoError)
                        .map(|v| {
                            (
                                x.strip_prefix(root)
                                    .unwrap_or_else(|_| x.as_path())
                                    .to_string(),
                                v,
                            )
                        })
                })
            })
            .collect::<Vec<tokio::task::JoinHandle<fpm::Result<(String, Vec<u8>)>>>>(),
    )
    .await
    .into_iter()
    .flatten()
    .flatten()
    .collect::<std::collections::BTreeMap<String, Vec<u8>>>();
    Ok(CloneResponse {
        package_name: config.package.name.to_string(),
        files,
        reserved_crs: config.get_reserved_crs(None).await?,
    })
}
