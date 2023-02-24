pub async fn create_package(root: &camino::Utf8Path) -> Result<(), fastn_cloud::CreateError> {
    let config = fastn_core::Config::read(Some(root.to_string()), false, None).await?;
    let (list_file, data_file) = create_tejar(root).await?;
    let list_content = read_to_string(list_file.as_path()).await?;
    let create_api_resp = create_api(
        config.package.name.as_str(),
        list_content.as_str(),
        r#"{"name": "Abrar Khan"}"#.to_string(),
    )
    .await?;
    let (missing_hashes_list, missing_hashes_content) = get_missing_checksums(
        list_content.as_str(),
        create_api_resp.missing_hashes.as_slice(),
        data_file.as_path(),
    )
    .await
    .unwrap();
    Ok(())
}

async fn get_missing_checksums(
    list_content: &str,
    missing_hashes: &[String],
    data_file: &camino::Utf8Path,
) -> Result<(String, Vec<u8>), ()> {
    let list = tejar::read::reader(list_content).unwrap().list;
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

#[derive(serde::Deserialize, Debug)]
pub struct CreateAPIResponse {
    pub key: String,
    pub missing_hashes: Vec<String>,
}

pub async fn create_api(
    package_name: &str,
    list_content: &str,
    meta_content: String,
) -> Result<CreateAPIResponse, fastn_cloud::http::PostError> {
    let list_bytes = list_content.to_string().into_bytes();
    let meta_bytes = meta_content.into_bytes();
    let query: std::collections::HashMap<_, _> = [
        ("package-name", package_name.to_string()),
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
    let response: CreateAPIResponse =
        fastn_cloud::http::post("/api/create/", body, &headers, &query).await?;
    println!("resp: {:?}", response);
    Ok(response)
}

pub async fn create_tejar(
    root: &camino::Utf8Path,
) -> Result<(camino::Utf8PathBuf, camino::Utf8PathBuf), tejar::error::CreateError> {
    let files: _ = fastn_cloud::utils::walkdir_util(root)
        .into_iter()
        .map(|file| tejar::create::InputFile {
            path: file.path.strip_prefix(&root).unwrap().to_path_buf(),
            content_type: file.content_type,
            gzip: file.gzip,
        })
        .collect::<Vec<_>>();
    tejar::create::create(&root, files.as_slice())
}

pub async fn read_to_string(path: &camino::Utf8Path) -> Result<String, tokio::io::Error> {
    // Let's keep this utility different for reading files
    tokio::fs::read_to_string(path).await
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
