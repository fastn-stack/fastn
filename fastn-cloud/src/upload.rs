#[derive(serde::Deserialize, Debug)]
pub struct MissingApiResponse {
    pub missing_hashes: Vec<String>,
}

pub async fn missing_files(
    cw_id: &str,
    list_content: &str,
    meta_content: String,
) -> Result<MissingApiResponse, fastn_cloud::http::PostError> {
    let list_bytes = list_content.to_string().into_bytes();
    let meta_bytes = meta_content.into_bytes();
    let query: std::collections::HashMap<_, _> = [
        ("cw-id", cw_id.to_string()),
        ("list-size", list_bytes.len().to_string()),
        ("meta-size", meta_bytes.len().to_string()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    let headers = [("Content-Type", "application/octet-stream")]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let body = list_bytes
        .into_iter()
        .chain(meta_bytes.into_iter())
        .collect::<Vec<u8>>();
    let response: MissingApiResponse =
        fastn_cloud::http::post("/api/missing-files/", body, &headers, &query).await?;
    Ok(response)
}

pub async fn upload(
    root: &camino::Utf8Path,
    sid: &str,
    cw_id: &str,
) -> Result<(), fastn_cloud::UploadError> {
    let (list_file, data_file) = fastn_cloud::create::tejar_create(root).await?;
    let list_content = tokio::fs::read_to_string(list_file.as_path()).await?;
    let missing_files_api_resp = missing_files(
        cw_id,
        list_content.as_str(),
        r#"{"name": "meta content todo"}"#.to_string(),
    )
    .await?;
    println!("Missing Files: {:?}", missing_files_api_resp);

    Ok(())
}

pub async fn _upload(sid: &str, cw_id: &str, list_content: &str, data: Vec<u8>) {}
