pub(crate) static GITHUB_PAGES_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"([^/]+)\.github\.io/([^/]+)").unwrap());

pub(crate) fn extract_github_details(pages_url: &str) -> Option<(String, String)> {
    if let Some(captures) = GITHUB_PAGES_REGEX.captures(pages_url) {
        let username = captures.get(1).unwrap().as_str().to_string();
        let repository = captures.get(2).unwrap().as_str().to_string();
        Some((username, repository))
    } else {
        None
    }
}

pub(crate) fn get_download_url_gh(username: &str, repository: &str) -> String {
    format!(
        "https://api.github.com/repos/{}/{}/zipball",
        username, repository
    )
}

pub(crate) fn get_package_source_url(package: &fastn_core::package::Package) -> Option<String> {
    if let Some((username, repository)) =
        fastn_update::utils::extract_github_details(package.name.as_str())
    {
        return Some(fastn_update::utils::get_download_url_gh(
            username.as_str(),
            repository.as_str(),
        ));
    }

    None
}

// https://api.github.com/repos/User/repo/:archive_format/:ref
// https://stackoverflow.com/questions/8377081/github-api-download-zip-or-tarball-link
pub(crate) async fn download_archive_from_gh(
    url: String,
) -> fastn_core::Result<(zip::ZipArchive<std::io::Cursor<Vec<u8>>>, String)> {
    use std::io::Seek;

    let zipball = fastn_core::http::http_get(&url).await?;
    let checksum = fastn_core::utils::generate_hash(zipball.clone());
    let mut zipball_cursor = std::io::Cursor::new(zipball);
    zipball_cursor.seek(std::io::SeekFrom::Start(0))?;
    let archive = zip::ZipArchive::new(zipball_cursor)?;
    Ok((archive, checksum))
}
