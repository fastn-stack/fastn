//! # Folder-Based Router Implementation

use crate::errors::FbrError;

/// Folder-based router for static files and templates
pub struct FolderBasedRouter {
    /// Path to the base directory (account or rig directory)
    base_path: std::path::PathBuf,
    /// Template engine for .fthml processing
    template_engine: Option<tera::Tera>,
}

impl FolderBasedRouter {
    /// Create new folder-based router for given directory
    pub fn new(base_path: impl Into<std::path::PathBuf>) -> Self {
        let base_path = base_path.into();

        // Initialize template engine for the public directory
        let template_engine = Self::init_template_engine(&base_path).ok();

        Self {
            base_path,
            template_engine,
        }
    }

    /// Initialize Tera template engine for the public directory
    fn init_template_engine(base_path: &std::path::Path) -> Result<tera::Tera, tera::Error> {
        let public_dir = base_path.join("public");
        let template_glob = public_dir.join("**").join("*.fthml");

        tracing::debug!("Initializing templates from: {}", template_glob.display());

        // Load all .fthml templates from public directory
        tera::Tera::new(&template_glob.to_string_lossy())
    }

    /// Route HTTP request to file or template with context
    pub async fn route_request(
        &self,
        request: &fastn_router::HttpRequest,
        context: Option<&crate::TemplateContext>,
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

        tracing::debug!(
            "FBR base_path: {}, public_dir: {}, file_path: {}",
            self.base_path.display(),
            public_dir.display(),
            file_path.display()
        );
        println!("🔍 FBR: Looking for file: {}", file_path.display());

        // Check if file exists
        if !file_path.exists() {
            println!("❌ FBR: File not found: {}", file_path.display());
            return Err(FbrError::FileNotFound {
                path: file_path.display().to_string(),
            });
        }

        println!("✅ FBR: File exists: {}", file_path.display());

        // Handle different file types
        if file_path.is_dir() {
            // Try index.html in directory
            let index_path = file_path.join("index.html");
            if index_path.exists() {
                self.serve_file(&index_path).await
            } else {
                Err(FbrError::FileNotFound {
                    path: file_path.display().to_string(),
                })
            }
        } else {
            // Serve file based on extension
            if file_path.extension().and_then(|e| e.to_str()) == Some("fthml") {
                self.serve_template(&file_path, context).await
            } else {
                self.serve_file(&file_path).await
            }
        }
    }

    /// Clean and normalize request path
    fn clean_path(&self, path: &str) -> Result<String, FbrError> {
        let cleaned = path.trim_start_matches('/');
        Ok(cleaned.to_string())
    }

    /// Serve static file
    async fn serve_file(
        &self,
        file_path: &std::path::Path,
    ) -> Result<fastn_router::HttpResponse, FbrError> {
        let content = tokio::fs::read(file_path)
            .await
            .map_err(|e| FbrError::FileReadFailed {
                path: file_path.display().to_string(),
                source: e,
            })?;

        // Detect MIME type from file extension
        let mime_type = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(crate::mime_type_for_extension)
            .unwrap_or("application/octet-stream");

        let mut response = fastn_router::HttpResponse::new(200, "OK");
        response
            .headers
            .insert("Content-Type".to_string(), mime_type.to_string());
        response = response.body(String::from_utf8_lossy(&content).to_string());

        Ok(response)
    }

    /// Serve .fthml template with context
    async fn serve_template(
        &self,
        template_path: &std::path::Path,
        context: Option<&crate::TemplateContext>,
    ) -> Result<fastn_router::HttpResponse, FbrError> {
        tracing::debug!("Processing .fthml template: {}", template_path.display());

        let template_engine = match &self.template_engine {
            Some(engine) => engine,
            None => {
                return Ok(
                    fastn_router::HttpResponse::new(500, "Internal Server Error")
                        .body("Template engine not initialized".to_string()),
                );
            }
        };

        // Get template name relative to public directory
        let public_dir = self.base_path.join("public");
        let template_name = template_path
            .strip_prefix(&public_dir)
            .map_err(|_| FbrError::InvalidPath {
                path: template_path.display().to_string(),
            })?
            .to_string_lossy()
            .to_string();

        // Create template context with custom functions
        let mut tera_context = match context {
            Some(ctx) => {
                // Make a mutable copy of the template engine to register functions
                let mut engine_copy = template_engine.clone();
                ctx.to_tera_context(&mut engine_copy)
            }
            None => tera::Context::new(),
        };

        // Add request context
        tera_context.insert("request_path", &template_path.display().to_string());
        tera_context.insert("timestamp", &chrono::Utc::now().timestamp());

        // Register functions and render template
        let rendered = if let Some(ctx) = context {
            // Clone engine and register functions
            let mut engine_with_functions = template_engine.clone();
            for (name, function) in &ctx.functions {
                engine_with_functions.register_function(name, *function);
            }
            engine_with_functions.render(&template_name, &tera_context)
        } else {
            template_engine.render(&template_name, &tera_context)
        };

        match rendered {
            Ok(rendered) => {
                let mut response = fastn_router::HttpResponse::new(200, "OK");
                response.headers.insert(
                    "Content-Type".to_string(),
                    "text/html; charset=utf-8".to_string(),
                );
                response = response.body(rendered);
                Ok(response)
            }
            Err(e) => {
                tracing::error!("Template rendering failed: {}", e);
                Err(FbrError::TemplateProcessingFailed {
                    template: template_name,
                    source: Box::new(e),
                })
            }
        }
    }
}
