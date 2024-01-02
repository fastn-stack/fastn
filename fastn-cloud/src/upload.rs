#[derive(serde::Deserialize, Debug)]
pub struct MissingApiResponse {
    pub missing_hashes: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Sid {
    pub sid: String,
}

pub async fn missing_files_api(
    sid: &str,
    cw_id: &str,
    list_content: &str,
    meta_content: String,
) -> Result<MissingApiResponse, fastn_cloud::http::Error> {
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
    let headers = [("Content-Type", "application/octet-stream"), ("sid", sid)]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let body = list_bytes
        .into_iter()
        .chain(meta_bytes.into_iter())
        .collect::<Vec<u8>>();
    let response: MissingApiResponse = fastn_cloud::http::request(
        reqwest::Method::POST,
        "/api/missing-files/",
        body,
        &headers,
        &query,
    )
    .await?;
    Ok(response)
}

pub async fn upload(
    ds: &fastn_ds::DocumentStore,
    root: &camino::Utf8Path,
    sid: &str,
    cw_id: &str,
) -> Result<(), fastn_cloud::UploadError> {
    let sid: Sid = serde_json::from_str(sid)?;
    let (list_file, data_file) = tejar_create(root).await?;
    let list_content = ds.read_to_string(list_file.as_path()).await.unwrap(); // Todo: Remove unwrap()
                                                                              // TODO: missing files handle sid
    println!("Getting Missing Files");
    let missing_files_api_resp = missing_files_api(
        sid.sid.as_str(),
        cw_id,
        list_content.as_str(),
        r#"{"name": "meta content todo"}"#.to_string(),
    )
    .await?;

    println!("Filtering Missing Hashes hashes");
    let (missing_hashes_list, missing_hashes_content) = get_missing_checksums(
        list_content.as_str(),
        missing_files_api_resp.missing_hashes.as_slice(),
        data_file.as_path(),
    )
    .await?;

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
) -> Result<UploadAPIResponse, fastn_cloud::http::Error> {
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
        fastn_cloud::http::request(reqwest::Method::PUT, "/api/upload/", body, &headers, &query)
            .await?;
    Ok(response)
}

pub async fn get_missing_checksums(
    list_content: &str,
    missing_hashes: &[String],
    data_file: &camino::Utf8Path,
) -> Result<(String, Vec<u8>), fastn_cloud::UploadError> {
    let list = tejar::read::reader(list_content)?.list;
    let mut data = vec![];
    let mut new_list = String::new();
    let mut offset = 0;
    for checksum in missing_hashes {
        let record = list.iter().find(|r| r.checksum.eq(checksum)).unwrap();
        let mut file_data =
            read_with_offset(data_file, record.offset as u64, record.file_size as usize)
                .await
                .unwrap();
        let list_record = tejar::create::ListRecord {
            data_file_name: record.data_file_name.to_string(),
            file_name: record.file_name.to_string(),
            content_type: record.content_type.to_string(),
            compression: "todo!".to_string(), // TODO:
            start: offset,
            size: record.file_size,
            timestamp: record.timestamp,
            checksum: record.checksum.to_string(),
        };
        offset += record.file_size;
        data.append(&mut file_data);
        new_list.push_str(list_record.to_string().as_str());
    }
    Ok((new_list, data))
}

pub async fn tejar_create(
    root: &camino::Utf8Path,
) -> Result<(camino::Utf8PathBuf, camino::Utf8PathBuf), tejar::error::CreateError> {
    let files = fastn_cloud::utils::walkdir_util(root)
        .into_iter()
        .filter(|f| !f.path.to_string().ends_with(".tejar-list"))
        .filter(|f| !f.path.to_string().ends_with(".tejar-data"))
        .map(|file| tejar::create::InputFile {
            path: file.path.strip_prefix(root).unwrap().to_path_buf(),
            content_type: file.content_type,
            gzip: file.gzip,
        })
        .collect::<Vec<_>>();
    tejar::create::create(root, files.as_slice())
}

pub async fn read_with_offset(
    path: &camino::Utf8Path,
    offset: u64,
    size: usize,
) -> Result<Vec<u8>, tokio::io::Error> {
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncSeekExt;
    let mut file = tokio::fs::File::open(path).await?;
    file.seek(tokio::io::SeekFrom::Start(offset)).await?;
    let mut buffer = vec![0u8; size];
    file.read_exact(&mut buffer).await?;
    Ok(buffer)
}
