use eyre::WrapErr;

pub const FASTN_LOCK: &str = "fastn.lock";
pub const MALAI_LOCK: &str = "malai.lock";

pub fn kulfi_lock_file(dir: &std::path::Path) -> eyre::Result<std::fs::File> {
    let path = dir.join(FASTN_LOCK);
    let file = std::fs::File::create(&path)
        .wrap_err_with(|| format!("failed to create lock file: {path:?}"))?;
    Ok(file)
}

pub fn malai_lock_file(dir: &std::path::Path) -> eyre::Result<std::fs::File> {
    let path = dir.join(MALAI_LOCK);
    let file = std::fs::File::create(&path)
        .wrap_err_with(|| format!("failed to create lock file: {path:?}"))?;
    Ok(file)
}

/// Acquire exclusive lock using standard library API
pub fn exclusive(lock_file: &std::fs::File) -> eyre::Result<()> {
    lock_file
        .try_lock()
        .wrap_err_with(|| "failed to take exclusive lock")
}

