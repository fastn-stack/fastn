#![allow(clippy::derive_partial_eq_without_eq)]
#![deny(unused_crate_dependencies)]

//! ASCII Rendering Engine for FTD Components
//! 
//! This crate provides ASCII art rendering for FTD components, enabling
//! terminal-friendly output and test-driven specification verification.

mod canvas;
mod layout;
mod renderer;
mod components;

pub use canvas::{Canvas, Position, Rect};
pub use layout::{AsciiLayout, LayoutConstraints, ComponentLayout};
pub use renderer::{AsciiRenderer, AsciiData};

/// Main entry point for ASCII rendering
pub fn render_ascii(compiled_doc: &fastn_compiler::CompiledDocument) -> String {
    let ascii_data = AsciiData::from_cd(compiled_doc);
    ascii_data.to_ascii()
}

/// Render a single .ftd file to ASCII (for testing)
pub fn render_ftd_file(path: &std::path::Path) -> Result<String, RenderError> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| RenderError::Io(e))?;
        
    let compiled = fastn_compiler::compile(
        &source,
        fastn_package::MainPackage::default(), // TODO: proper package
        None,
    ).consume_with_fn(/* TODO: definition provider */);
    
    match compiled {
        Ok(doc) => Ok(render_ascii(&doc)),
        Err(e) => Err(RenderError::Compilation(format!("{:?}", e))),
    }
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