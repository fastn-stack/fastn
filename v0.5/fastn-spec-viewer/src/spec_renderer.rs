/// Specification rendering using clean DocumentRenderer API
use crate::embedded_specs;
use fastn_ansi_renderer::DocumentRenderer;

/// High-level spec rendering that uses embedded specs + clean renderer API
pub fn render_spec(spec_name: &str, width: usize, height: usize) -> Result<SpecOutput, Box<dyn std::error::Error>> {
    // Get embedded spec source (spec-viewer responsibility)  
    let document_source = embedded_specs::get_embedded_spec(spec_name)?;
    
    // Use clean DocumentRenderer API (pure rendering)
    let rendered = DocumentRenderer::render_from_source(&document_source, width, height)?;
    
    Ok(SpecOutput {
        ansi_version: rendered.to_ansi().to_string(),
        plain_version: rendered.to_plain(), 
        side_by_side: rendered.to_side_by_side(),
    })
}

/// Generate all dimensions for a specification
pub fn render_all_dimensions(spec_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut all_sections = Vec::new();
    
    // Generate all three dimensions
    for (width, height) in [(40, 64), (80, 128), (120, 192)] {
        let spec_output = render_spec(spec_name, width, height)?;
        
        // Create section with strict formatting: exactly 4 newlines before header, 1 after
        let section = if all_sections.is_empty() {
            format!("# {}x{}\n\n{}\n\n\n\n", width, height, spec_output.side_by_side)
        } else {
            format!("\n\n\n\n# {}x{}\n\n{}\n\n\n\n", width, height, spec_output.side_by_side)
        };
        all_sections.push(section);
    }
    
    Ok(all_sections.join(""))
}

/// Specification output (wrapper around DocumentRenderer output)
#[derive(Debug, Clone)]
pub struct SpecOutput {
    pub ansi_version: String,
    pub plain_version: String,
    pub side_by_side: String,
}

impl SpecOutput {
    /// For terminal display
    pub fn terminal_display(&self) -> &str {
        &self.ansi_version
    }
    
    /// For editor viewing
    pub fn editor_display(&self) -> &str {
        &self.plain_version
    }
    
    /// For spec file format
    pub fn spec_file_format(&self) -> &str {
        &self.side_by_side
    }
}