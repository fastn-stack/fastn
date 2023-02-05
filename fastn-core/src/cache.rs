// New cache
// functions get, update, update and get, get and update, get or init
// all should be atomic operation

// For Now we can get file name update the value

// TODO: Need to change it later
// TODO: https://stackoverflow.com/questions/29445026/converting-number-primitives-i32-f64-etc-to-byte-representations

// TODO: what is this lock protecting? We already have a lock in fastn_core::commands::serve::LOCK.
static LOCK: once_cell::sync::Lazy<async_lock::RwLock<()>> =
    once_cell::sync::Lazy::new(|| async_lock::RwLock::new(()));

/*pub async fn get(path: &str) -> fastn_core::Result<usize> {
    match LOCK.try_read() {
        Ok(_) => {
            let value = tokio::fs::read_to_string(path).await?;
            Ok(value.parse()?)
        }
        Err(e) => Err(fastn_core::Error::GenericError(e.to_string())),
    }
}

pub async fn create(path: &str) -> fastn_core::Result<usize> {
    use tokio::io::AsyncWriteExt;
    match LOCK.try_write() {
        Ok(_) => {
            let content: usize = 1;
            tokio::fs::File::create(path)
                .await?
                .write_all(content.to_string().as_bytes())
                .await?;
            Ok(_get_without_lock(path).await?)
        }
        Err(e) => Err(fastn_core::Error::GenericError(e.to_string())),
    }
}

pub async fn increment(path: &str) -> fastn_core::Result<usize> {
    update_get(path, 1).await
}

pub async fn create_or_inc(path: &str) -> fastn_core::Result<usize> {
    if camino::Utf8Path::new(path).exists() {
        increment(path).await
    } else {
        create(path).await
    }
}*/

async fn _get_without_lock(path: &str) -> fastn_core::Result<usize> {
    let value = tokio::fs::read_to_string(path).await?;
    Ok(value.parse()?)
}

async fn _create_without_lock(path: &str) -> fastn_core::Result<usize> {
    use tokio::io::AsyncWriteExt;
    let content: usize = 1;
    tokio::fs::File::create(path)
        .await?
        .write_all(content.to_string().as_bytes())
        .await?;
    _get_without_lock(path).await
}

async fn update_get(path: &str, value: usize) -> fastn_core::Result<usize> {
    // TODO: why are we not just taking the lock using `let _lock = LOCK.write()`?
    match LOCK.try_write() {
        Some(_lock) => {
            let old_value = _get_without_lock(path).await?;
            tokio::fs::write(path, (old_value + value).to_string().as_bytes()).await?;
            Ok(_get_without_lock(path).await?)
        }
        None => Err(fastn_core::Error::GenericError(
            "Failed to acquire lock".to_string(),
        )),
    }
}

async fn update_create(path: &str, value: usize) -> fastn_core::Result<usize> {
    // TODO: why are we not just taking the lock using `let _lock = LOCK.write()`?
    match LOCK.try_write() {
        Some(_lock) => {
            let old_value = _create_without_lock(path).await?;
            tokio::fs::write(path, (old_value + value).to_string().as_bytes()).await?;
            Ok(_get_without_lock(path).await?)
        }
        None => Err(fastn_core::Error::GenericError(
            "Failed to acquire lock".to_string(),
        )),
    }
}

pub async fn update(path: &str, value: usize) -> fastn_core::Result<usize> {
    if camino::Utf8Path::new(path).exists() {
        update_get(path, value).await
    } else {
        update_create(path, value).await
    }
}
