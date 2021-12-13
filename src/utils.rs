pub fn get_timestamp_nanosecond() -> u128 {
    match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_nanos(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

pub fn history_path(id: &str, base_path: &str, timestamp: &u128) -> camino::Utf8PathBuf {
    let id_with_timestamp_extension = if let Some((id, ext)) = id.rsplit_once('.') {
        format!("{}.{}.{}", id, timestamp, ext)
    } else {
        format!("{}.{}", id, timestamp)
    };
    let base_path = camino::Utf8PathBuf::from(base_path);
    base_path.join(".history").join(id_with_timestamp_extension)
}

pub fn track_path(id: &str, base_path: &str) -> camino::Utf8PathBuf {
    let base_path = camino::Utf8PathBuf::from(base_path);
    base_path.join(".tracks").join(format!("{}.track", id))
}
