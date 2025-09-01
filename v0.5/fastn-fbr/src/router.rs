//! # Folder-Based Router Implementation

use crate::errors::FbrError;

/// Folder-based router for static files and templates
pub struct FolderBasedRouter {
    /// Path to the base directory (account or rig directory)
    base_path: std::path::PathBuf,
}

impl FolderBasedRouter {
    /// Create new folder-based router for given directory
    pub fn new(base_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// Route HTTP request to file or template
    pub async fn route_request(
        &self,
        request: &fastn_router::HttpRequest,
    ) -> Result<fastn_router::HttpResponse, FbrError> {
        tracing::debug!("FBR routing: {} {}", request.method, request.path);

        // Only handle GET requests for now
        if request.method != "GET" {
            return Ok(fastn_router::HttpResponse::new(405, "Method Not Allowed")
                .body("Only GET requests supported".to_string()));
        }

        // Clean and validate path
        let clean_path = self.clean_path(&request.path)?;
        
        // Check for directory traversal attacks
        if clean_path.contains("..") || clean_path.starts_with('/') {
            return Err(FbrError::InvalidPath {
                path: clean_path.clone(),
            });
        }

        // Construct file path in public directory
        let public_dir = self.base_path.join("public");
        let file_path = if clean_path.is_empty() {
            public_dir.join("index.html")
        } else {
            public_dir.join(&clean_path)
        };

        tracing::debug!("FBR file path: {}", file_path.display());

        // Check if file exists
        if !file_path.exists() {
            return Err(FbrError::FileNotFound {
                path: file_path.display().to_string(),
            });
        }

        // Handle different file types
        if file_path.is_dir() {
            // Try index.html in directory
            let index_path = file_path.join("index.html");
            if index_path.exists() {
                return self.serve_file(&index_path).await;
            } else {
                return Err(FbrError::FileNotFound {
                    path: file_path.display().to_string(),
                });
            }
        } else {
            // Serve file based on extension
            if file_path.extension().and_then(|e| e.to_str()) == Some("fthml") {
                return self.serve_template(&file_path).await;
            } else {
                return self.serve_file(&file_path).await;
            }
        }
    }

    /// Clean and normalize request path
    fn clean_path(&self, path: &str) -> Result<String, FbrError> {
        let cleaned = path.trim_start_matches('/');
        Ok(cleaned.to_string())
    }

    /// Serve static file
    async fn serve_file(&self, file_path: &std::path::Path) -> Result<fastn_router::HttpResponse, FbrError> {
        let content = tokio::fs::read(file_path).await.map_err(|e| {
            FbrError::FileReadFailed {
                path: file_path.display().to_string(),
                source: e,
            }
        })?;

        // Detect MIME type from file extension
        let mime_type = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(crate::mime_type_for_extension)
            .unwrap_or("application/octet-stream");

        let mut response = fastn_router::HttpResponse::new(200, "OK");
        response.headers.insert("Content-Type".to_string(), mime_type.to_string());
        response = response.body(String::from_utf8_lossy(&content).to_string());

        Ok(response)
    }

    /// Serve .fthml template (placeholder)
    async fn serve_template(&self, template_path: &std::path::Path) -> Result<fastn_router::HttpResponse, FbrError> {
        tracing::info!("Processing .fthml template: {}", template_path.display());

        // TODO: Implement FTD template processing
        // For now, serve as plain HTML
        let content = tokio::fs::read_to_string(template_path).await.map_err(|e| {
            FbrError::FileReadFailed {
                path: template_path.display().to_string(),
                source: e,
            }
        })?;

        let processed_content = format!(
            "<!-- Processed .fthml template: {} -->\n\
            <html><head><title>FTD Template</title></head><body>\n\
            <h1>FTD Template (Processing TODO)</h1>\n\
            <pre>{}</pre>\n\
            </body></html>",
            template_path.display(),
            html_escape::encode_text(&content)
        );

        let mut response = fastn_router::HttpResponse::new(200, "OK");
        response.headers.insert("Content-Type".to_string(), "text/html; charset=utf-8".to_string());
        response = response.body(processed_content);

        Ok(response)
    }

}