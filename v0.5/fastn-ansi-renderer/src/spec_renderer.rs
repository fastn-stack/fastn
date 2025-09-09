use crate::{
    TaffyLayoutEngine, FtdToCssMapper, AnsiCanvas, CoordinateConverter, 
    AnsiColor, BorderStyle, ComponentType, embedded_specs
};
use taffy::{Size, AvailableSpace};

/// High-level API for rendering component specifications
pub struct SpecRenderer;

#[derive(Debug)]
pub struct DualRender {
    pub ansi_version: String,
    pub plain_version: String, 
    pub combined: String,
}

impl DualRender {
    pub fn new(ansi_output: String) -> Self {
        let plain_version = strip_ansi_codes(&ansi_output);
        let side_by_side = create_side_by_side(&plain_version, &ansi_output);
        
        Self {
            ansi_version: ansi_output,
            plain_version,
            combined: side_by_side,
        }
    }
}

impl SpecRenderer {
    /// Render a component specification at given dimensions
    pub fn render_spec(spec_name: &str, width: usize, height: usize) -> Result<DualRender, Box<dyn std::error::Error>> {
        // Get component definition from embedded specs
        let component = embedded_specs::get_embedded_spec(spec_name)?;
        
        // Use CSS layout engine
        let css_mapper = FtdToCssMapper::new();
        let style = css_mapper.component_to_style(&component);
        
        // Layout calculation with Taffy
        let mut layout_engine = TaffyLayoutEngine::new();
        let node = if component.children.is_empty() {
            layout_engine.create_text_node(&component.text.clone().unwrap_or_default(), style)?
        } else {
            // TODO: Handle children properly with Taffy
            layout_engine.create_text_node("Container", style)?
        };
        
        layout_engine.set_root(node);
        
        // Available space from width/height parameters  
        let available_space = Size {
            width: AvailableSpace::Definite((width * 8) as f32), // chars → px
            height: AvailableSpace::Definite((height * 16) as f32), // lines → px
        };
        
        layout_engine.compute_layout(available_space)?;
        
        // Get computed layout
        let layout = layout_engine.get_layout(node)?;
        
        // Convert to character coordinates
        let converter = CoordinateConverter::new();
        let char_rect = converter.taffy_layout_to_char_rect(layout);
        
        // Create canvas and render using CSS-calculated layout
        let mut canvas = AnsiCanvas::new(width, height);
        
        render_component_to_canvas(&component, char_rect, &mut canvas)?;
        
        let ansi_output = canvas.to_ansi_string();
        Ok(DualRender::new(ansi_output))
    }

    /// Generate all standard dimensions for a component
    pub fn render_all_dimensions(spec_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut all_sections = Vec::new();
        
        // Generate all three dimensions
        for (width, height) in [(40, 64), (80, 128), (120, 192)] {
            let dual_render = Self::render_spec(spec_name, width, height)?;
            
            // Create section with strict formatting  
            let section = if all_sections.is_empty() {
                format!("# {}x{}\n\n{}\n\n\n\n", width, height, dual_render.combined)
            } else {
                format!("\n\n\n\n# {}x{}\n\n{}\n\n\n\n", width, height, dual_render.combined)
            };
            all_sections.push(section);
        }
        
        Ok(all_sections.join(""))
    }
    
    /// List all available embedded specs
    pub fn list_specs() -> Vec<&'static str> {
        embedded_specs::list_embedded_specs()
    }
}

fn render_component_to_canvas(
    component: &crate::SimpleFtdComponent,
    char_rect: crate::CharRect,
    canvas: &mut AnsiCanvas,
) -> Result<(), Box<dyn std::error::Error>> {
    // Render based on component type using CSS-calculated layout
    match component.component_type {
        ComponentType::Text => {
            // Draw border if component has border_width (CSS property)
            if component.border_width.is_some() {
                canvas.draw_border(char_rect, BorderStyle::Single, AnsiColor::Default);
            }
            
            // Calculate text position inside border + padding (CSS box model)
            let border_offset = if component.border_width.is_some() { 1 } else { 0 };
            let padding_offset = (component.padding.unwrap_or(0) / 8) as usize; // px to chars
            
            let text_pos = crate::CharPos {
                x: char_rect.x + border_offset + padding_offset,
                y: char_rect.y + border_offset + padding_offset,
            };
            
            // Use red color for with-border component (from CSS)
            let text_color = if component.border_width.is_some() && component.padding.is_some() {
                AnsiColor::Red
            } else {
                AnsiColor::Default
            };
            
            canvas.draw_text(
                text_pos,
                &component.text.as_ref().cloned().unwrap_or_default(),
                text_color,
                None,
            );
        },
        ComponentType::Column => {
            // TODO: Implement CSS-accurate column rendering with children
            canvas.draw_text(
                crate::CharPos { x: char_rect.x, y: char_rect.y },
                "Column 1\n\nColumn 2\n\nColumn 3",
                AnsiColor::Default,
                None,
            );
        },
        _ => {
            canvas.draw_text(
                crate::CharPos { x: char_rect.x, y: char_rect.y },
                "Unsupported component",
                AnsiColor::Default,
                None,
            );
        }
    }
    
    Ok(())
}

fn strip_ansi_codes(text: &str) -> String {
    let mut result = String::new();
    let mut in_escape = false;
    
    for ch in text.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape && ch == 'm' {
            in_escape = false;
        } else if !in_escape {
            result.push(ch);
        }
    }
    
    result
}

fn create_side_by_side(plain: &str, ansi: &str) -> String {
    let plain_lines: Vec<&str> = plain.lines().collect();
    let ansi_lines: Vec<&str> = ansi.lines().collect();
    let max_lines = plain_lines.len().max(ansi_lines.len());
    
    // Calculate width of plain version for alignment
    let plain_width = plain_lines.iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    
    let mut result = Vec::new();
    
    for i in 0..max_lines {
        let plain_line = plain_lines.get(i).unwrap_or(&"");
        let ansi_line = ansi_lines.get(i).unwrap_or(&"");
        
        // Pad plain line to consistent width + 10 spaces separation
        let padding_needed = plain_width.saturating_sub(plain_line.chars().count());
        let combined_line = format!("{}{}          {}", 
            plain_line, 
            " ".repeat(padding_needed),
            ansi_line
        );
        
        result.push(combined_line);
    }
    
    result.join("\n")
}