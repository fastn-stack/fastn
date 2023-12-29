pub async fn read(
    path: impl AsRef<std::path::Path> + std::fmt::Debug,
) -> fastn_core::Result<Vec<u8>> {
    let path_str = format!("{:#?}", path);
    tokio::fs::read(path)
        .await
        .map_err(|e| fastn_core::Error::FastnIoError {
            io_error: e,
            path: path_str,
        })
}

pub async fn read_to_string(
    path: impl AsRef<std::path::Path> + std::fmt::Debug,
) -> fastn_core::Result<String> {
    let path_str = format!("{:#?}", path);
    tokio::fs::read_to_string(path)
        .await
        .map_err(|e| fastn_core::Error::FastnIoError {
            io_error: e,
            path: path_str,
        })
}
