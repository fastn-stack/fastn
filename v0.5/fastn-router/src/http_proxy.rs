//! # HTTP Proxy Types
//!
//! Types for proxying HTTP requests over P2P connections (following kulfi/malai pattern).

/// HTTP request for P2P transmission (following kulfi pattern)
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProxyRequest {
    pub uri: String,
    pub method: String,
    pub headers: Vec<(String, Vec<u8>)>,
}

impl From<hyper::http::request::Parts> for ProxyRequest {
    fn from(r: hyper::http::request::Parts) -> Self {
        let mut headers = vec![];
        for (k, v) in r.headers {
            let k = match k {
                Some(v) => v.to_string(),
                None => continue,
            };
            headers.push((k, v.as_bytes().to_vec()));
        }

        ProxyRequest {
            uri: r.uri.to_string(),
            method: r.method.to_string(),
            headers,
        }
    }
}

/// HTTP response from P2P transmission
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ProxyResponse {
    pub status: u16,
    pub headers: Vec<(String, Vec<u8>)>,
}

/// Proxy data for protocol header extra field
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ProxyData {
    /// HTTP request proxy
    Http { target_id52: String },
}