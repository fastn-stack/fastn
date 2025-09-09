/// Clean document rendering API - no spec knowledge, just renders fastn documents

use crate::{TaffyLayoutEngine, FtdToCssMapper, SimpleFtdComponent, AnsiCanvas, CoordinateConverter};
use taffy::{Size, AvailableSpace};

/// Pure document rendering - takes fastn document, returns ANSI output
pub struct DocumentRenderer;

impl DocumentRenderer {
    /// Render a fastn document at given dimensions
    pub fn render_document(document: &FastnDocument, width: usize, height: usize) -> Result<String, Box<dyn std::error::Error>> {
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
        
        Self::render_component_to_canvas(&document.root_component, char_rect, &mut canvas)?;
        
        Ok(canvas.to_ansi_string())
    }

    /// Parse fastn source and render (convenience method)
    pub fn render_from_source(source: &str, width: usize, height: usize) -> Result<String, Box<dyn std::error::Error>> {
        let document = parse_fastn_source(source)?;
        Self::render_document(&document, width, height)
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