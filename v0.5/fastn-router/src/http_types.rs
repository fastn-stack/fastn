//! # HTTP Types
//!
//! Basic HTTP request/response types for fastn web interface.
//! 
//! Note: fastn-router provides routing for FTD documents and WASM modules.
//! These types are for general web application HTTP handling (account/rig interfaces).

/// HTTP request representation
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub host: String,
    pub headers: std::collections::HashMap<String, String>,
}

/// HTTP response representation  
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    /// Create new HTTP response
    pub fn new(status: u16, status_text: &str) -> Self {
        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "text/plain; charset=utf-8".to_string());
        headers.insert("Connection".to_string(), "close".to_string());
        
        Self {
            status,
            status_text: status_text.to_string(),
            headers,
            body: String::new(),
        }
    }
    
    /// Set response body
    pub fn body(mut self, body: String) -> Self {
        self.headers.insert("Content-Length".to_string(), body.len().to_string());
        self.body = body;
        self
    }
    
    /// Convert to HTTP response string
    pub fn to_http_string(&self) -> String {
        let mut response = format!("HTTP/1.1 {} {}\r\n", self.status, self.status_text);
        
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        
        response.push_str("\r\n");
        response.push_str(&self.body);
        
        response
    }
    
    /// Create 200 OK response
    pub fn ok(body: String) -> Self {
        Self::new(200, "OK").body(body)
    }
    
    /// Create 404 Not Found response  
    pub fn not_found(message: String) -> Self {
        Self::new(404, "Not Found").body(message)
    }
    
    /// Create 500 Internal Server Error response
    pub fn internal_error(message: String) -> Self {
        Self::new(500, "Internal Server Error").body(message)
    }
}