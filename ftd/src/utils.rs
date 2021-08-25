pub fn has_extension(id: &str) -> bool {
    std::path::Path::new(id).extension().is_some()
}

pub fn if_true<F1: FnOnce() -> U, F2: FnOnce() -> U, U>(c: bool, a: F1, b: F2) -> U {
    if c {
        a()
    } else {
        b()
    }
}
