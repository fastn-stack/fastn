use colorize::AnsiColor;
use itertools::Itertools;

macro_rules! warning {
    ($s:expr,) => {
        warning!($s)
    };
    ($s:expr) => {
        println!("{}", format!("{}", $s).yellow());
    };
}

pub fn print_end(msg: &str, start: std::time::Instant) {
    if fpm::utils::is_test() {
        println!("done in <omitted>");
    } else {
        println!(
            // TODO: instead of lots of spaces put proper erase current terminal line thing
            "\r{} in {:?}.                          ",
            msg.to_string().green(),
            start.elapsed()
        );
    }
}

pub trait HasElements {
    fn has_elements(&self) -> bool;
}

impl<T> HasElements for Vec<T> {
    fn has_elements(&self) -> bool {
        !self.is_empty()
    }
}

pub(crate) fn timestamp_nanosecond() -> u128 {
    match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_nanos(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

pub(crate) fn language_to_human(language: &str) -> String {
    realm_lang::Language::from_2_letter_code(language)
        .map(|v| v.human())
        .unwrap_or_else(|_| language.to_string())
}

pub(crate) fn nanos_to_rfc3339(nanos: &u128) -> String {
    let time = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(*nanos as u64);
    chrono::DateTime::<chrono::Utc>::from(time).to_rfc3339()
}

pub(crate) fn history_path(id: &str, base_path: &str, timestamp: &u128) -> camino::Utf8PathBuf {
    let id_with_timestamp_extension = snapshot_id(id, timestamp);
    let base_path = camino::Utf8PathBuf::from(base_path);
    base_path.join(".history").join(id_with_timestamp_extension)
}

pub(crate) fn snapshot_id(path: &str, timestamp: &u128) -> String {
    if let Some((id, ext)) = path.rsplit_once('.') {
        format!("{}.{}.{}", id, timestamp, ext)
    } else {
        format!("{}.{}", path, timestamp)
    }
}

pub(crate) fn track_path(id: &str, base_path: &str) -> camino::Utf8PathBuf {
    let base_path = camino::Utf8PathBuf::from(base_path);
    base_path.join(".tracks").join(format!("{}.track", id))
}

pub(crate) async fn get_number_of_documents(config: &fpm::Config) -> fpm::Result<String> {
    let mut no_of_docs = fpm::snapshot::get_latest_snapshots(&config.root)
        .await?
        .len()
        .to_string();
    if let Ok(original_path) = config.original_path() {
        let no_of_original_docs = fpm::snapshot::get_latest_snapshots(&original_path)
            .await?
            .len();
        no_of_docs = format!("{} / {}", no_of_docs, no_of_original_docs);
    }
    Ok(no_of_docs)
}

pub(crate) fn get_extension(file_name: &str) -> fpm::Result<String> {
    if let Some((_, ext)) = file_name.rsplit_once('.') {
        return Ok(ext.to_string());
    }
    Err(fpm::Error::UsageError {
        message: format!("extension not found, `{}`", file_name),
    })
}

pub(crate) async fn get_current_document_last_modified_on(
    config: &fpm::Config,
    document_id: &str,
) -> Option<String> {
    fpm::snapshot::get_latest_snapshots(&config.root)
        .await
        .unwrap_or_default()
        .get(document_id)
        .map(nanos_to_rfc3339)
}

pub(crate) async fn get_last_modified_on(path: &camino::Utf8PathBuf) -> Option<String> {
    fpm::snapshot::get_latest_snapshots(path)
        .await
        .unwrap_or_default()
        .values()
        .into_iter()
        .max()
        .map(nanos_to_rfc3339)
}

/*
// todo get_package_title needs to be implemented
    @amitu need to come up with idea
    This data would be used in fpm.title
pub(crate) fn get_package_title(config: &fpm::Config) -> String {
    let fpm = if let Ok(fpm) = std::fs::read_to_string(config.root.join("index.ftd")) {
        fpm
    } else {
        return config.package.name.clone();
    };
    let lib = fpm::Library {
        config: config.clone(),
        markdown: None,
        document_id: "index.ftd".to_string(),
        translated_data: Default::default(),
        current_package: std::sync::Arc::new(std::sync::Mutex::new(vec![config.package.clone()])),
    };
    let main_ftd_doc = match ftd::p2::Document::from("index.ftd", fpm.as_str(), &lib) {
        Ok(v) => v,
        Err(_) => {
            return config.package.name.clone();
        }
    };
    match &main_ftd_doc.title() {
        Some(x) => x.rendered.clone(),
        _ => config.package.name.clone(),
    }
}*/

#[async_recursion::async_recursion(?Send)]
pub async fn copy_dir_all(
    src: impl AsRef<std::path::Path> + 'static,
    dst: impl AsRef<std::path::Path> + 'static,
) -> std::io::Result<()> {
    tokio::fs::create_dir_all(&dst).await?;
    let mut dir = tokio::fs::read_dir(src).await?;
    while let Some(child) = dir.next_entry().await? {
        if child.metadata().await?.is_dir() {
            copy_dir_all(child.path(), dst.as_ref().join(child.file_name())).await?;
        } else {
            tokio::fs::copy(child.path(), dst.as_ref().join(child.file_name())).await?;
        }
    }
    Ok(())
}

pub(crate) fn seconds_to_human(s: u64) -> String {
    let days = s / 3600 / 24;
    let hours = s / 3600 - days * 24;
    let months = days / 30;
    if s == 0 {
        "Just now".to_string()
    } else if s == 1 {
        "One second ago".to_string()
    } else if s < 60 {
        format!("{} seconds ago", s)
    } else if s < 3600 {
        format!("{} minutes ago", s / 60)
    } else if s < 3600 * 10 {
        let r = s - hours * 60;
        if r == 0 {
            format!("{} hours ago", hours)
        } else if hours == 1 && r == 1 {
            "An hour and a minute ago".to_string()
        } else if hours == 1 {
            format!("An hour and {} minutes ago", r)
        } else {
            format!("{} hours ago", hours)
        }
    } else if days < 1 {
        format!("{} hours ago", hours)
    } else if days == 1 && hours == 0 {
        "A day ago".to_string()
    } else if days == 1 && hours == 1 {
        "A day an hour ago".to_string()
    } else if days == 1 {
        format!("A day ago and {} hours ago", hours)
    } else if days < 7 && hours == 0 {
        format!("{} days ago", days)
    } else if months == 1 {
        "A month ago".to_string()
    } else if months < 24 {
        format!("{} months ago", months)
    } else {
        format!("{} years ago", months / 12)
    }
}

pub(crate) fn validate_base_url(package: &fpm::Package) -> fpm::Result<()> {
    if package.base.is_none() {
        warning!("expected base in fpm.package");
    }

    Ok(())
}

#[allow(dead_code)]
pub fn escape_ftd(file: &str) -> String {
    file.split('\n')
        .map(|v| {
            if v.starts_with("-- ") || v.starts_with("--- ") {
                format!("\\{}", v)
            } else {
                v.to_string()
            }
        })
        .join("\n")
}

pub(crate) fn id_to_path(id: &str) -> String {
    id.replace("/index.ftd", "/")
        .replace("index.ftd", "/")
        .replace(".ftd", std::path::MAIN_SEPARATOR.to_string().as_str())
        .replace("/index.md", "/")
        .replace("/README.md", "/")
        .replace("index.md", "/")
        .replace("README.md", "/")
        .replace(".md", std::path::MAIN_SEPARATOR.to_string().as_str())
}

pub(crate) fn replace_markers(
    s: &str,
    config: &fpm::Config,
    main_id: &str,
    title: &str,
    base_url: &str,
    main_rt: &ftd::Document,
) -> String {
    s.replace("__ftd_doc_title__", title)
        .replace(
            "__ftd_canonical_url__",
            config.package.generate_canonical_url(main_id).as_str(),
        )
        .replace("__ftd_js__", fpm::ftd_js().as_str())
        .replace("__ftd_body_events__", main_rt.body_events.as_str())
        .replace("__ftd_css__", fpm::ftd_css())
        .replace("__ftd_element_css__", main_rt.css_collector.as_str())
        .replace("__fpm_js__", fpm::fpm_js())
        .replace(
            "__ftd_data_main__",
            fpm::font::escape(
                serde_json::to_string_pretty(&main_rt.data)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .as_str(),
        )
        .replace(
            "__ftd_external_children_main__",
            fpm::font::escape(
                serde_json::to_string_pretty(&main_rt.external_children)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .as_str(),
        )
        .replace(
            "__main__",
            format!("{}{}", main_rt.html, config.get_font_style(),).as_str(),
        )
        .replace("__base_url__", base_url)
}

pub fn is_test() -> bool {
    std::env::args().any(|e| e == "--test")
}

pub(crate) fn url_regex() -> regex::Regex {
    regex::Regex::new(
        r#"((([A-Za-z]{3,9}:(?://)?)(?:[-;:&=\+\$,\w]+@)?[A-Za-z0-9.-]+|(?:www.|[-;:&=\+\$,\w]+@)[A-Za-z0-9.-]+)((?:/[\+~%/.\w_]*)?\??(?:[-\+=&;%@.\w_]*)\#?(?:[\w]*))?)"#
    ).unwrap()
}

pub(crate) async fn construct_url_and_get_str(url: &str) -> fpm::Result<String> {
    construct_url_and_return_response(url.to_string(), |f| async move {
        http_get_str(f.as_str()).await
    })
    .await
}

pub(crate) async fn construct_url_and_get(url: &str) -> fpm::Result<Vec<u8>> {
    construct_url_and_return_response(
        url.to_string(),
        |f| async move { http_get(f.as_str()).await },
    )
    .await
}

pub(crate) async fn construct_url_and_return_response<T, F, D>(url: String, f: F) -> fpm::Result<D>
where
    F: FnOnce(String) -> T + Copy,
    T: futures::Future<Output = std::result::Result<D, fpm::Error>> + Send + 'static,
{
    if url[1..].contains("://") || url.starts_with("//") {
        f(url).await
    } else if let Ok(response) = f(format!("https://{}", url)).await {
        Ok(response)
    } else {
        f(format!("http://{}", url)).await
    }
}

pub(crate) async fn http_get<T: reqwest::IntoUrl + std::fmt::Debug>(
    url: T,
) -> fpm::Result<Vec<u8>> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("fpm"),
    );
    let c = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let url_f = format!("{:?}", url);
    let mut res = c.get(url).send()?;
    if !res.status().eq(&reqwest::StatusCode::OK) {
        return Err(fpm::Error::APIResponseError(format!(
            "url: {}, response_status: {}, response: {:?}",
            url_f,
            res.status(),
            res.text()
        )));
    }
    let mut buf: Vec<u8> = vec![];
    res.copy_to(&mut buf)?;
    Ok(buf)
}

pub(crate) async fn http_get_str<T: reqwest::IntoUrl + std::fmt::Debug>(
    url: T,
) -> fpm::Result<String> {
    let url_f = format!("{:?}", url);
    match http_get(url).await {
        Ok(bytes) => String::from_utf8(bytes).map_err(|e| fpm::Error::UsageError {
            message: format!(
                "Cannot convert the response to string: URL: {:?}, ERROR: {}",
                url_f, e
            ),
        }),
        Err(e) => Err(e),
    }
}

pub(crate) async fn write(
    root: &camino::Utf8PathBuf,
    file_path: &str,
    data: &[u8],
) -> fpm::Result<()> {
    if root.join(file_path).exists() {
        return Ok(());
    }
    update(root, file_path, data).await
}

pub(crate) async fn update(
    root: &camino::Utf8PathBuf,
    file_path: &str,
    data: &[u8],
) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;

    let (file_root, file_name) = if let Some((file_root, file_name)) = file_path.rsplit_once('/') {
        (file_root.to_string(), file_name.to_string())
    } else {
        ("".to_string(), file_path.to_string())
    };

    if !root.join(&file_root).exists() {
        tokio::fs::create_dir_all(root.join(&file_root)).await?;
    }

    Ok(
        tokio::fs::File::create(root.join(file_root).join(file_name))
            .await?
            .write_all(data)
            .await?,
    )
}
