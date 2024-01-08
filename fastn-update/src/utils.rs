pub(crate) async fn download_archive(
    url: String,
) -> fastn_core::Result<zip::ZipArchive<std::io::Cursor<Vec<u8>>>> {
    use std::io::Seek;

    let zipball = fastn_core::http::http_get(&url).await?;
    let mut zipball_cursor = std::io::Cursor::new(zipball);
    zipball_cursor.seek(std::io::SeekFrom::Start(0))?;
    let archive = zip::ZipArchive::new(zipball_cursor)?;
    Ok(archive)
}
