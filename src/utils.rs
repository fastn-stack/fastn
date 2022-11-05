pub trait ValueOf {
    fn value_of_(&self, name: &str) -> Option<&str>;
}

impl ValueOf for clap::ArgMatches {
    fn value_of_(&self, name: &str) -> Option<&str> {
        self.get_one::<String>(name).map(|v| v.as_str())
    }
}

// https://stackoverflow.com/questions/71985357/whats-the-best-way-to-write-a-custom-format-macro
#[macro_export]
macro_rules! warning {
    ($($t:tt)*) => {{
        use colored::Colorize;
        let msg = format!($($t)*);
        eprintln!("WARN: {}", msg.yellow());
        msg
    }};
}

pub fn print_end(msg: &str, start: std::time::Instant) {
    use colored::Colorize;

    if fpm::utils::is_test() {
        println!("done in <omitted>");
    } else {
        println!(
            // TODO: instead of lots of spaces put proper erase current terminal line thing
            "\r{} in {:?}.                          ",
            msg.green(),
            start.elapsed()
        );
    }
}

pub fn time(msg: &str) -> Timer {
    Timer {
        start: std::time::Instant::now(),
        msg,
    }
}

pub struct Timer<'a> {
    start: std::time::Instant,
    msg: &'a str,
}

impl<'a> Timer<'a> {
    pub fn it<T>(&self, a: T) -> T {
        use colored::Colorize;

        if !fpm::utils::is_test() {
            let duration = format!("{:?}", self.start.elapsed());
            println!("{} in {}", self.msg.green(), duration.red());
        }

        a
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
    nanos.to_string() // TODO
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
    src: impl AsRef<camino::Utf8Path> + 'static,
    dst: impl AsRef<camino::Utf8Path> + 'static,
) -> std::io::Result<()> {
    tokio::fs::create_dir_all(dst.as_ref()).await?;
    let mut dir = tokio::fs::read_dir(src.as_ref()).await?;
    while let Some(child) = dir.next_entry().await? {
        if child.metadata().await?.is_dir() {
            copy_dir_all(
                camino::Utf8PathBuf::from_path_buf(child.path())
                    .expect("we only work with utf8 paths"),
                dst.as_ref().join(
                    child
                        .file_name()
                        .into_string()
                        .expect("we only work with utf8 paths"),
                ),
            )
            .await?;
        } else {
            tokio::fs::copy(
                child.path(),
                dst.as_ref().join(
                    child
                        .file_name()
                        .into_string()
                        .expect("we only work with utf8 paths"),
                ),
            )
            .await?;
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
    if package.download_base_url.is_none() {
        warning!("expected base in fpm.package: {:?}", package.name);
    }

    Ok(())
}

#[allow(dead_code)]
pub fn escape_ftd(file: &str) -> String {
    use itertools::Itertools;

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

pub fn id_to_path(id: &str) -> String {
    id.replace("/index.ftd", "/")
        .replace("index.ftd", "/")
        .replace(".ftd", std::path::MAIN_SEPARATOR.to_string().as_str())
        .replace("/index.md", "/")
        .replace("/README.md", "/")
        .replace("index.md", "/")
        .replace("README.md", "/")
        .replace(".md", std::path::MAIN_SEPARATOR.to_string().as_str())
}

/// returns true if an existing file named "file_name"
/// exists in the root package folder
fn is_file_in_root(root: &str, file_name: &str) -> bool {
    camino::Utf8PathBuf::from(root).join(file_name).exists()
}

/// returns favicon html tag as string
/// (if favicon is passed as header in fpm.package or if any favicon.* file is present in the root package folder)
/// otherwise returns None
fn resolve_favicon(
    root_path: &str,
    package_name: &str,
    favicon: &Option<String>,
) -> Option<String> {
    /// returns html tag for using favicon.
    fn favicon_html(favicon_path: &str, content_type: &str) -> String {
        let favicon_html = format!(
            "\n<link rel=\"shortcut icon\" href=\"{}\" type=\"{}\">",
            favicon_path, content_type
        );
        favicon_html
    }

    /// returns relative favicon path from package and its mime content type
    fn get_favicon_path_and_type(package_name: &str, favicon_path: &str) -> (String, String) {
        // relative favicon path wrt package
        let path = camino::Utf8PathBuf::from(package_name).join(favicon_path);
        // mime content type of the favicon
        let content_type = mime_guess::from_path(path.as_str()).first_or_octet_stream();

        (path.to_string(), content_type.to_string())
    }

    // favicon image path from fpm.package if provided
    let fav_path = favicon;

    let (full_fav_path, fav_mime_content_type): (String, String) = {
        match fav_path {
            Some(ref path) => {
                // In this case, favicon is provided with fpm.package in FPM.ftd
                get_favicon_path_and_type(package_name, path)
            }
            None => {
                // If favicon not provided so we will look for favicon in the package directory
                // By default if any file favicon.* is present we will use that file instead
                // In case of favicon.* conflict priority will be: .ico > .svg > .png > .jpg.
                // Default searching directory being the root folder of the package

                // Just check if any favicon exists in the root package directory
                // in the above mentioned priority order
                let found_favicon_id = if is_file_in_root(root_path, "favicon.ico") {
                    "favicon.ico"
                } else if is_file_in_root(root_path, "favicon.svg") {
                    "favicon.svg"
                } else if is_file_in_root(root_path, "favicon.png") {
                    "favicon.png"
                } else if is_file_in_root(root_path, "favicon.jpg") {
                    "favicon.jpg"
                } else {
                    // Not using any favicon
                    return None;
                };

                get_favicon_path_and_type(package_name, found_favicon_id)
            }
        }
    };

    // Will use some favicon
    Some(favicon_html(&full_fav_path, &fav_mime_content_type))
}

pub fn replace_markers(
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
        .replace(
            "__favicon_html_tag__",
            resolve_favicon(
                config.root.as_str(),
                config.package.name.as_str(),
                &config.package.favicon,
            )
            .unwrap_or_default()
            .as_str(),
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

pub(crate) async fn write(
    root: &camino::Utf8PathBuf,
    file_path: &str,
    data: &[u8],
) -> fpm::Result<()> {
    if root.join(file_path).exists() {
        return Ok(());
    }
    update1(root, file_path, data).await
}

// TODO: remove this function use update instead
pub(crate) async fn update1(
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

pub(crate) async fn copy(
    from: impl AsRef<camino::Utf8Path>,
    to: impl AsRef<camino::Utf8Path>,
) -> fpm::Result<()> {
    let content = tokio::fs::read(from.as_ref()).await?;
    fpm::utils::update(to, content.as_slice()).await
}

pub(crate) async fn update(root: impl AsRef<camino::Utf8Path>, data: &[u8]) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;

    let (file_root, file_name) = if let Some(file_root) = root.as_ref().parent() {
        (
            file_root,
            root.as_ref()
                .file_name()
                .ok_or_else(|| fpm::Error::UsageError {
                    message: format!(
                        "Invalid File Path: Can't find file name `{:?}`",
                        root.as_ref()
                    ),
                })?,
        )
    } else {
        return Err(fpm::Error::UsageError {
            message: format!(
                "Invalid File Path: file path doesn't have parent: {:?}",
                root.as_ref()
            ),
        });
    };

    if !file_root.exists() {
        tokio::fs::create_dir_all(file_root).await?;
    }

    Ok(tokio::fs::File::create(file_root.join(file_name))
        .await?
        .write_all(data)
        .await?)
}

pub(crate) fn ids_matches(id1: &str, id2: &str) -> bool {
    return strip_id(id1).eq(&strip_id(id2));

    fn strip_id(id: &str) -> String {
        let id = id
            .trim()
            .replace("/index.html", "/")
            .replace("index.html", "/");
        if id.eq("/") {
            return id;
        }
        id.trim_matches('/').to_string()
    }
}

/// Parse argument from CLI
/// If CLI command: fpm serve --identities a@foo.com,foo
/// key: --identities -> output: a@foo.com,foo
pub fn parse_from_cli(key: &str) -> Option<String> {
    use itertools::Itertools;
    let args = std::env::args().collect_vec();
    let mut index = None;
    for (idx, arg) in args.iter().enumerate() {
        if arg.eq(key) {
            index = Some(idx);
        }
    }
    index
        .and_then(|idx| args.get(idx + 1))
        .map(String::to_string)
}

/// Remove path: It can be directory or file
pub async fn remove(path: &std::path::Path) -> std::io::Result<()> {
    if path.is_file() {
        tokio::fs::remove_file(path).await?;
    } else if path.is_dir() {
        tokio::fs::remove_dir_all(path).await?
    } else if path.is_symlink() {
        // TODO:
        // It can be a directory or a file
    }
    Ok(())
}

/// Remove from provided `root` except given list
pub async fn remove_except(root: &camino::Utf8Path, except: &[&str]) -> fpm::Result<()> {
    use itertools::Itertools;
    let except = except
        .iter()
        .map(|x| root.join(x))
        .map(|x| x.into_std_path_buf())
        .collect_vec();
    let mut all = tokio::fs::read_dir(root).await?;
    while let Some(file) = all.next_entry().await? {
        if except.contains(&file.path()) {
            continue;
        }
        if file.metadata().await?.is_dir() {
            tokio::fs::remove_dir_all(file.path()).await?;
        } else if file.metadata().await?.is_file() {
            tokio::fs::remove_file(file.path()).await?;
        }
    }
    Ok(())
}

/// /api/?a=1&b=2&c=3 => vec[(a, 1), (b, 2), (c, 3)]
pub fn query(uri: &str) -> fpm::Result<Vec<(String, String)>> {
    use itertools::Itertools;
    Ok(
        url::Url::parse(format!("https://fifthtry.com/{}", uri).as_str())?
            .query_pairs()
            .into_owned()
            .collect_vec(),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn query() {
        assert_eq!(
            super::query("/api/?a=1&b=2&c=3").unwrap(),
            vec![
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "2".to_string()),
                ("c".to_string(), "3".to_string())
            ]
        )
    }
}

pub fn ignore_headers() -> Vec<&'static str> {
    vec!["host", "x-forwarded-ssl"]
}
