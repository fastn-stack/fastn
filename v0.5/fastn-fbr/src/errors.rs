//! # Error Types for Folder-Based Router

use thiserror::Error;

/// Error type for folder-based routing operations
#[derive(Error, Debug)]
pub enum FbrError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Directory not found: {path}")]
    DirectoryNotFound { path: String },

    #[error("Failed to read file: {path}")]
    FileReadFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid path: {path}")]
    InvalidPath { path: String },

    #[error("Template processing failed: {template}")]
    TemplateProcessingFailed { 
        template: String,
        #[source] 
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("MIME type detection failed: {extension}")]
    MimeTypeDetectionFailed { extension: String },
}