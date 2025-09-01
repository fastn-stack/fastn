//! # fastn-fbr: Folder-Based Router
//!
//! Provides folder-based routing for static files and templates.
//!
//! ## Features
//! - Static file serving from `public/` directories
//! - .fthml template processing and rendering
//! - Clean folder-based routing with automatic MIME type detection
//! - Integration with fastn-account and fastn-rig for web UIs
//!
//! ## Usage
//! ```ignore
//! let router = FolderBasedRouter::new("/path/to/account");
//! let response = router.route_request(&request).await?;
//! ```
//!
//! ## Directory Structure
//! ```ignore
//! account_or_rig_directory/
//! ├── public/           # Static files and templates  
//! │   ├── index.html
//! │   ├── /-/mail/      # Email UI
//! │   │   ├── inbox.fthml
//! │   │   └── compose.fthml
//! │   └── assets/       # CSS, JS, images
//! └── src/              # Source content (copied to public/)
//! ```

extern crate self as fastn_fbr;

mod errors;
mod router;

pub use errors::*;
pub use router::FolderBasedRouter;

/// MIME type detection for file extensions
pub fn mime_type_for_extension(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "fthml" => "text/html; charset=utf-8", // FTD HTML templates
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" | "woff2" => "font/woff",
        "ttf" => "font/ttf",
        "txt" => "text/plain; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",
        _ => "application/octet-stream",
    }
}
