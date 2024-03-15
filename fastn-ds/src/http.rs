fn default_client_builder() -> reqwest::Client {
    reqwest::ClientBuilder::default().build().unwrap()
}

pub static DEFAULT_CLIENT: once_cell::sync::Lazy<std::sync::Arc<reqwest::Client>> =
    once_cell::sync::Lazy::new(|| std::sync::Arc::new(default_client_builder()));
