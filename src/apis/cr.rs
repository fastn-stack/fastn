#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
pub struct CreateCRRequest {
    pub title: Option<String>,
}

pub async fn create_cr(req: CreateCRRequest) -> fpm::Result<fpm::http::Response> {
    match create_cr_worker(req).await {
        Ok(cr_number) => {
            #[derive(serde::Serialize)]
            struct CreateCRResponse {
                url: String,
            }
            let url = format!("-/{}/-/about/", cr_number);
            fpm::http::api_ok(CreateCRResponse { url })
        }
        Err(err) => fpm::http::api_error(err.to_string()),
    }
}

async fn create_cr_worker(cr_request: CreateCRRequest) -> fpm::Result<usize> {
    let config = fpm::Config::read(None, false).await?;
    let cr_number = config.extract_cr_number().await?;
    let default_title = format!("CR#{cr_number}");
    let cr_meta = fpm::cr::CRMeta {
        title: cr_request.title.unwrap_or(default_title),
        cr_number: cr_number as usize,
        open: true,
    };
    fpm::commands::create_cr::add_cr_to_workspace(&config, &cr_meta).await?;
    Ok(cr_number as usize)
}

pub async fn create_cr_page() -> fpm::Result<fpm::http::Response> {
    match create_cr_page_worker().await {
        Ok(body) => Ok(fpm::http::ok(body)),
        Err(err) => fpm::http::api_error(err.to_string()),
    }
}

async fn create_cr_page_worker() -> fpm::Result<Vec<u8>> {
    let mut config = fpm::Config::read(None, false).await?;
    let create_cr_ftd = fpm::package_info_create_cr(&config)?;

    let main_document = fpm::Document {
        id: "create-cr.ftd".to_string(),
        content: create_cr_ftd,
        parent_path: config.root.as_str().to_string(),
        package_name: config.package.name.clone(),
    };

    fpm::package_doc::read_ftd(&mut config, &main_document, "/", false).await
}
