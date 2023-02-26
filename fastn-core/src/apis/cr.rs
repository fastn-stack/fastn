#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
pub struct CreateCRRequest {
    pub title: Option<String>,
}

pub async fn create_cr(
    req: &fastn_core::http::Request,
    cr_req: CreateCRRequest,
) -> fastn_core::Result<fastn_core::http::Response> {
    match create_cr_worker(req, cr_req).await {
        Ok(cr_number) => {
            #[derive(serde::Serialize)]
            struct CreateCRResponse {
                url: String,
            }
            let url = format!("-/{}/-/about/", cr_number);
            fastn_core::http::api_ok(CreateCRResponse { url })
        }
        Err(err) => fastn_core::http::api_error(err.to_string()),
    }
}

async fn create_cr_worker(
    req: &fastn_core::http::Request,
    cr_request: CreateCRRequest,
) -> fastn_core::Result<usize> {
    let config = fastn_core::Config::read(None, false, Some(req)).await?;
    let cr_number = config.extract_cr_number().await?;
    let default_title = format!("CR#{cr_number}");
    let cr_meta = fastn_core::cr::CRMeta {
        title: cr_request.title.unwrap_or(default_title),
        cr_number: cr_number as usize,
        open: true,
    };
    fastn_core::commands::create_cr::add_cr_to_workspace(&config, &cr_meta).await?;
    Ok(cr_number as usize)
}

pub async fn create_cr_page(
    req: fastn_core::http::Request,
) -> fastn_core::Result<fastn_core::http::Response> {
    match create_cr_page_worker(req).await {
        Ok(body) => Ok(fastn_core::http::ok(body)),
        Err(err) => fastn_core::http::api_error(err.to_string()),
    }
}

async fn create_cr_page_worker(req: fastn_core::http::Request) -> fastn_core::Result<Vec<u8>> {
    let mut config = fastn_core::Config::read(None, false, Some(&req)).await?;
    let create_cr_ftd = fastn_core::package_info_create_cr(&config)?;

    let main_document = fastn_core::Document {
        id: "create-cr.ftd".to_string(),
        content: create_cr_ftd,
        parent_path: config.root.as_str().to_string(),
        package_name: config.package.name.clone(),
    };

    fastn_core::package::package_doc::read_ftd(&mut config, &main_document, "/", false).await
}
