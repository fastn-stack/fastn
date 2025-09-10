/// Clean document rendering API - no spec knowledge, just renders fastn documents

use crate::{TaffyLayoutEngine, FtdToCssMapper, SimpleFtdComponent, AnsiCanvas, CoordinateConverter};
use taffy::{Size, AvailableSpace};

/// Rendered output with multiple format options
#[derive(Debug, Clone)]
pub struct Rendered {
    ansi_output: String,
}

impl Rendered {
    pub fn new(ansi_output: String) -> Self {
        Self { ansi_output }
    }
    
    /// Get ANSI version with escape codes for terminal display
    pub fn to_ansi(&self) -> &str {
        &self.ansi_output
    }
    
    /// Get plain ASCII version with ANSI codes stripped  
    pub fn to_plain(&self) -> String {
        strip_ansi_codes(&self.ansi_output)
    }
    
    /// Get side-by-side format for specification files
    pub fn to_side_by_side(&self) -> String {
        let plain = self.to_plain();
        create_side_by_side(&plain, &self.ansi_output)
    }
}

/// Pure document rendering - takes fastn document, returns structured output
pub struct DocumentRenderer;

impl DocumentRenderer {
    /// Render a fastn document at given dimensions
    pub fn render_document(document: &FastnDocument, width: usize, height: usize) -> Result<Rendered, Box<dyn std::error::Error>> {
        // Use CSS layout engine
        let css_mapper = FtdToCssMapper::new();
        let style = css_mapper.component_to_style(&document.root_component);
        
        // Layout calculation with Taffy
        let mut layout_engine = TaffyLayoutEngine::new();
        let node = layout_engine.create_text_node(
            &document.root_component.text.as_ref().cloned().unwrap_or_default(), 
            style
        )?;
        
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
        
        // Add outer window border using full requested dimensions
        let window_rect = crate::CharRect {
            x: 0,
            y: 0,
            width: width,  // Use full requested width
            height: height, // Use full requested height
        };
        
        // Use double border style for outer window
        canvas.draw_border(window_rect, crate::BorderStyle::Double, crate::AnsiColor::Default);
        
        // Offset component position to be inside window border
        let component_rect = crate::CharRect {
            x: char_rect.x + 2, // Inside window border + margin
            y: char_rect.y + 2, // Inside window border + margin  
            width: char_rect.width,
            height: char_rect.height,
        };
        
        Self::render_component_to_canvas(&document.root_component, component_rect, &mut canvas)?;
        
        Ok(Rendered::new(canvas.to_ansi_string()))
    }

    /// Parse fastn source and render at exact dimensions
    pub fn render_from_source(source: &str, width: usize, height: usize) -> Result<Rendered, Box<dyn std::error::Error>> {
        let document = parse_fastn_source(source)?;
        
        // Use exact requested dimensions - no calculation override
        Self::render_document(&document, width, height)
    }

    /// Calculate intelligent height based on document content
    fn calculate_content_height(document: &FastnDocument, _width: usize) -> usize {
        let component = &document.root_component;
        
        // Base height for text content
        let mut height: usize = 1; // Text line
        
        // Add border height
        if component.border_width.is_some() {
            height += 2; // Top + bottom border
        }
        
        // Add padding height  
        if let Some(padding_px) = component.padding {
            height += ((padding_px / 8) * 2) as usize; // Top + bottom padding (px to chars)
        }
        
        // Add outer window margin
        height += 4; // Outer window padding
        
        // Minimum useful height
        height.max(5)
    }

    fn render_component_to_canvas(
        component: &SimpleFtdComponent,
        char_rect: crate::CharRect,
        canvas: &mut AnsiCanvas,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::{ComponentType, AnsiColor, BorderStyle, CharPos};
        
        match component.component_type {
            ComponentType::Text => {
                // Draw border if component has border_width (CSS property)
                if component.border_width.is_some() {
                    canvas.draw_border(char_rect, BorderStyle::Single, AnsiColor::Default);
                }
                
                // Calculate text position inside border + padding (CSS box model)
                let border_offset = if component.border_width.is_some() { 1 } else { 0 };
                let padding_offset = (component.padding.unwrap_or(0) / 8) as usize; // px to chars
                
                let text_pos = CharPos {
                    x: char_rect.x + border_offset + padding_offset,
                    y: char_rect.y + border_offset + padding_offset,
                };
                
                // TODO: Get color from CSS properties instead of hardcoded logic
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
                // TODO: Implement CSS-accurate column rendering with Taffy children
                canvas.draw_text(
                    CharPos { x: char_rect.x, y: char_rect.y },
                    "Column Layout",
                    AnsiColor::Default,
                    None,
                );
            },
            _ => {
                canvas.draw_text(
                    CharPos { x: char_rect.x, y: char_rect.y },
                    "Document Element",
                    AnsiColor::Default,
                    None,
                );
            }
        }
        
        Ok(())
    }
}

/// Represents a parsed fastn document
#[derive(Debug, Clone)]
pub struct FastnDocument {
    pub root_component: SimpleFtdComponent,
    // TODO: Add more document-level properties (imports, variables, etc.)
}

/// Simple fastn document parser (placeholder - will integrate with real fastn parser)
fn parse_fastn_source(source: &str) -> Result<FastnDocument, Box<dyn std::error::Error>> {
    // Simple parsing for now - will integrate with actual fastn parser
    if source.contains("-- ftd.text:") {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            if let Some(text_start) = line.find("-- ftd.text:") {
                let text_content = line[text_start + 12..].trim();
                
                let mut component = SimpleFtdComponent::text(text_content);
                
                // Parse CSS properties from source
                if source.contains("border-width") {
                    component = component.with_border(1);
                }
                if source.contains("padding") {
                    component = component.with_padding(8);
                }
                
                return Ok(FastnDocument {
                    root_component: component,
                });
            }
        }
    }
    
    Err("Unsupported fastn document".into())
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