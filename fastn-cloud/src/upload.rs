#[derive(serde::Deserialize, Debug)]
pub struct MissingApiResponse {
    pub missing_hashes: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Sid {
    pub sid: String,
}

pub async fn missing_files_api(
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
    let sid: Sid = serde_json::from_str(sid)?;
    let (list_file, data_file) = fastn_cloud::create::tejar_create(root).await?;
    let list_content = tokio::fs::read_to_string(list_file.as_path()).await?;
    // TODO: missing files handle sid
    println!("Getting Missing Files");
    let missing_files_api_resp = missing_files_api(
        cw_id,
        list_content.as_str(),
        r#"{"name": "meta content todo"}"#.to_string(),
    )
    .await?;

    println!("Filtering Missing Hashes hashes");
    let (missing_hashes_list, missing_hashes_content) = fastn_cloud::create::get_missing_checksums(
        list_content.as_str(),
        missing_files_api_resp.missing_hashes.as_slice(),
        data_file.as_path(),
    )
    .await
    .unwrap();

    println!("Uploading Missing List Content");
    let resp = upload_api(
        sid.sid.as_str(),
        cw_id,
        missing_hashes_list.as_str(),
        missing_hashes_content,
    )
    .await?;

    println!("Content Uploaded: {:?}", resp);

    Ok(())
}

#[derive(serde::Deserialize, Debug)]
pub struct UploadAPIResponse {
    pub domain: String,
}

pub async fn upload_api(
    sid: &str,
    cw_id: &str,
    list_content: &str,
    data: Vec<u8>,
) -> Result<UploadAPIResponse, fastn_cloud::http::PostError> {
    let list_bytes = list_content.to_string().into_bytes();
    let query: std::collections::HashMap<_, _> = [
        ("cw-id", cw_id.to_string()),
        ("list-size", list_bytes.len().to_string()),
        ("data-size", data.len().to_string()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    let headers = [("Content-Type", "application/octet-stream"), ("sid", sid)]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let body = list_bytes
        .into_iter()
        .chain(data.into_iter())
        .collect::<Vec<u8>>();
    let response: UploadAPIResponse =
        fastn_cloud::http::put("/api/upload/", body, &headers, &query).await?;
    Ok(response)
}
