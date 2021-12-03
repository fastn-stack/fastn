pub fn get_timestamp_nanosecond() -> u128 {
    match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_nanos(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

#[cfg(test)]
pub fn has_extension(id: &str) -> bool {
    std::path::Path::new(id).extension().is_some()
}

#[cfg(test)]
pub fn if_true<F1: FnOnce() -> U, F2: FnOnce() -> U, U>(c: bool, a: F1, b: F2) -> U {
    if c {
        a()
    } else {
        b()
    }
}
