use fastn_ascii_renderer::{Canvas, Position, Rect};
use fastn_ascii_renderer::components::TextRenderer;
use fastn_ascii_renderer::{ComponentRenderer, LayoutConstraints};

#[test]
fn test_canvas_basic() {
    let mut canvas = Canvas::new(10, 5);
    canvas.draw_text(Position { x: 0, y: 0 }, "Hello", None);
    
    let output = canvas.to_string();
    assert!(output.contains("Hello"));
}

#[test]
fn test_text_with_border() {
    let text_renderer = TextRenderer::new("Test".to_string())
        .with_border(1)
        .with_padding(2);
    
    let constraints = LayoutConstraints::default();
    let layout = text_renderer.calculate_layout(&constraints);
    
    // Text: 4 chars + padding: 4 + border: 2 = 10 total width
    assert_eq!(layout.width, 10);
    // Text: 1 line + padding: 4 + border: 2 = 7 total height
    assert_eq!(layout.height, 7);
}

#[test]
fn test_text_wrapping() {
    let text_renderer = TextRenderer::new("This is a long text".to_string())
        .with_width(10);
    
    let constraints = LayoutConstraints::default();
    let layout = text_renderer.calculate_layout(&constraints);
    
    // Should wrap to multiple lines
    assert!(layout.content_height > 1);
}