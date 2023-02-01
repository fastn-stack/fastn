#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
pub struct CreateCRRequest {
    pub title: Option<String>,
}

pub async fn create_cr(
    req: &fastn::http::Request,
    cr_req: CreateCRRequest,
) -> fastn::Result<fastn::http::Response> {
    match create_cr_worker(req, cr_req).await {
        Ok(cr_number) => {
            #[derive(serde::Serialize)]
            struct CreateCRResponse {
                url: String,
            }
            let url = format!("-/{}/-/about/", cr_number);
            fastn::http::api_ok(CreateCRResponse { url })
        }
        Err(err) => fastn::http::api_error(err.to_string()),
    }
}

async fn create_cr_worker(
    req: &fastn::http::Request,
    cr_request: CreateCRRequest,
) -> fastn::Result<usize> {
    let config = fastn::Config::read(None, false, Some(req)).await?;
    let cr_number = config.extract_cr_number().await?;
    let default_title = format!("CR#{cr_number}");
    let cr_meta = fastn::cr::CRMeta {
        title: cr_request.title.unwrap_or(default_title),
        cr_number: cr_number as usize,
        open: true,
    };
    fastn::commands::create_cr::add_cr_to_workspace(&config, &cr_meta).await?;
    Ok(cr_number as usize)
}

pub async fn create_cr_page(req: fastn::http::Request) -> fastn::Result<fastn::http::Response> {
    match create_cr_page_worker(req).await {
        Ok(body) => Ok(fastn::http::ok(body)),
        Err(err) => fastn::http::api_error(err.to_string()),
    }
}

async fn create_cr_page_worker(req: fastn::http::Request) -> fastn::Result<Vec<u8>> {
    let mut config = fastn::Config::read(None, false, Some(&req)).await?;
    let create_cr_ftd = fastn::package_info_create_cr(&config)?;

    let main_document = fastn::Document {
        id: "create-cr.ftd".to_string(),
        content: create_cr_ftd,
        parent_path: config.root.as_str().to_string(),
        package_name: config.package.name.clone(),
    };

    fastn::package::package_doc::read_ftd(&mut config, &main_document, "/", false).await
}
