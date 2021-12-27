macro_rules! warning {
    ($s:expr,) => {
        warning!($s)
    };
    ($s:expr) => {
        use std::io::Write;
        use termcolor::WriteColor;

        let mut stdout = termcolor::StandardStream::stdout(termcolor::ColorChoice::Always);
        stdout.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)))?;

        writeln!(&mut stdout, "{}", $s)?;
        stdout.reset()?;
    };
}

pub trait HasElements {
    fn has_elements(&self) -> bool;
}

impl<T> HasElements for Vec<T> {
    fn has_elements(&self) -> bool {
        !self.is_empty()
    }
}

pub(crate) fn get_timestamp_nanosecond() -> u128 {
    match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_nanos(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

pub(crate) fn history_path(id: &str, base_path: &str, timestamp: &u128) -> camino::Utf8PathBuf {
    let id_with_timestamp_extension = if let Some((id, ext)) = id.rsplit_once('.') {
        format!("{}.{}.{}", id, timestamp, ext)
    } else {
        format!("{}.{}", id, timestamp)
    };
    let base_path = camino::Utf8PathBuf::from(base_path);
    base_path.join(".history").join(id_with_timestamp_extension)
}

pub(crate) fn track_path(id: &str, base_path: &str) -> camino::Utf8PathBuf {
    let base_path = camino::Utf8PathBuf::from(base_path);
    base_path.join(".tracks").join(format!("{}.track", id))
}

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

pub(crate) fn validate_zip_url(package: &fpm::Package) -> fpm::Result<()> {
    let zip = if let Some(ref zip) = package.zip {
        zip
    } else {
        warning!("expected zip in fpm.package");
        return Ok(());
    };
    let output = std::process::Command::new("git")
        .args(["remote", "get-url", "--push", "origin"])
        .output()
        .unwrap();
    if output.status.success() {
        let github_repo_name = std::str::from_utf8(&output.stdout)
            .unwrap()
            .trim()
            .replace(".git", "")
            .replace("git@github.com:", "")
            .replace("https://github.com/", "")
            .to_lowercase();
        let expected_zip_url = format!(
            "github.com/{}/archive/refs/heads/main.zip",
            github_repo_name
        );
        if &expected_zip_url != zip {
            let warning_message = format!("warning: valid `zip` is expected in fpm.package.\nsuggestion: change `zip` value to '{}'\n", expected_zip_url);
            warning!(warning_message);
        }
    }

    Ok(())
}
