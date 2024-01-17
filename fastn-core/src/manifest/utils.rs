static GITHUB_PAGES_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"([^/]+)\.github\.io/([^/]+)").unwrap());

fn extract_github_details(package_name: String) -> Option<(String, String)> {
    if let Some(captures) = GITHUB_PAGES_REGEX.captures(package_name.as_str()) {
        let username = captures.get(1).unwrap().as_str().to_string();
        let repository = captures.get(2).unwrap().as_str().to_string();
        Some((username, repository))
    } else {
        None
    }
}

// https://api.github.com/repos/User/repo/:archive_format/:ref
// https://stackoverflow.com/questions/8377081/github-api-download-zip-or-tarball-link
pub(crate) fn get_gh_zipball_url(package_name: String) -> Option<String> {
    if let Some((username, repository)) = extract_github_details(package_name) {
        let url = format!(
            "https://codeload.github.com/{}/{}/zip/refs/heads/main",
            username, repository
        );

        return Some(url);
    }

    None
}
