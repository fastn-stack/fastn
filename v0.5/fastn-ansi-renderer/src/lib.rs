#![allow(clippy::derive_partial_eq_without_eq)]
#![deny(unused_crate_dependencies)]

//! ASCII Rendering Engine for FTD Components
//! 
//! This crate provides ASCII art rendering for FTD components, enabling
//! terminal-friendly output and test-driven specification verification.

// Placeholder usage for dependencies (will be used in later phases)
use ansi_term as _;
use unicode_width as _;

mod canvas;
mod layout;
mod renderer;
pub mod components;
mod taffy_integration;
mod ftd_types;
mod css_mapper;
mod ansi_canvas;
pub mod document_renderer;

// spec_viewer module moved to separate fastn-spec-viewer crate

pub use canvas::{Canvas, Position, Rect};
pub use layout::{AsciiLayout, LayoutConstraints, ComponentLayout};
pub use renderer::{AsciiRenderer, AsciiData, ComponentRenderer};
pub use taffy_integration::TaffyLayoutEngine;
pub use ftd_types::{SimpleFtdComponent, ComponentType, FtdSize};
pub use css_mapper::FtdToCssMapper;
pub use ansi_canvas::{AnsiCanvas, AnsiColor, CharPos, CharRect, CoordinateConverter, BorderStyle};
pub use document_renderer::{DocumentRenderer, FastnDocument, Rendered};

/// Main entry point for ASCII rendering (placeholder for now)
pub fn render_ascii(_compiled_doc: &str) -> String {
    "<!-- ASCII Rendering Placeholder -->".to_string()
}

/// Render a single .ftd file to ASCII (for testing)
pub fn render_ftd_file(path: &std::path::Path) -> Result<String, RenderError> {
    let _source = std::fs::read_to_string(path)
        .map_err(RenderError::Io)?;
        
    // TODO: Implement proper FTD file rendering
    // This is a placeholder until the full compilation pipeline is ready
    Ok(format!("<!-- FTD file: {} -->", path.display()))
}

/// Verify .ftd file against .ftd-rendered expected output
pub fn verify_rendering(ftd_path: &std::path::Path, expected_path: &std::path::Path) -> Result<(), TestError> {
    let actual = render_ftd_file(ftd_path)?;
    let expected = std::fs::read_to_string(expected_path)?;
    
    if actual.trim() == expected.trim() {
        Ok(())
    } else {
        Err(TestError::OutputMismatch {
            expected: expected.clone(),
            actual,
            ftd_file: ftd_path.to_path_buf(),
        })
    }
}

#[derive(Debug)]
pub enum RenderError {
    Io(std::io::Error),
    Compilation(String),
}

#[derive(Debug)]
pub enum TestError {
    Render(RenderError),
    OutputMismatch {
        expected: String,
        actual: String,
        ftd_file: std::path::PathBuf,
    },
}

impl From<RenderError> for TestError {
    fn from(e: RenderError) -> Self {
        TestError::Render(e)
    }
}

impl From<std::io::Error> for TestError {
    fn from(e: std::io::Error) -> Self {
        TestError::Render(RenderError::Io(e))
    }
}