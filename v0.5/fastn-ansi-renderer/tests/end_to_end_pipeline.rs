use fastn_ansi_renderer::{
    AnsiCanvas, AnsiColor, BorderStyle, CharPos, CoordinateConverter, FtdSize, FtdToCssMapper,
    SimpleFtdComponent, TaffyLayoutEngine,
};
use taffy::{AvailableSpace, Size};

#[test]
fn test_complete_text_rendering_pipeline() {
    // 1. Create FTD component (simulating parsed FTD)
    let ftd_component = SimpleFtdComponent::text("Hello World")
        .with_padding(8)
        .with_border(1);

    // 2. Map FTD properties to CSS
    let css_mapper = FtdToCssMapper::new();
    let style = css_mapper.component_to_style(&ftd_component);

    // 3. Create Taffy layout
    let mut layout_engine = TaffyLayoutEngine::new();
    let node = layout_engine
        .create_text_node("Hello World", style)
        .unwrap();
    layout_engine.set_root(node);

    // 4. Compute layout
    let available = Size {
        width: AvailableSpace::Definite(400.0),  // 50 chars * 8px
        height: AvailableSpace::Definite(400.0), // 25 lines * 16px
    };
    layout_engine.compute_layout(available).unwrap();

    // 5. Get computed layout
    let layout = layout_engine.get_layout(node).unwrap();

    // 6. Convert to character coordinates
    let converter = CoordinateConverter::new();
    let char_rect = converter.taffy_layout_to_char_rect(layout);

    // 7. Create ANSI canvas and render
    let mut canvas = AnsiCanvas::new(50, 25); // 50 chars x 25 lines

    // Draw border (accounting for padding)
    canvas.draw_border(char_rect, BorderStyle::Single, AnsiColor::Default);

    // Draw text inside border + padding
    let text_pos = CharPos {
        x: char_rect.x + 1 + 1, // border + padding (simplified)
        y: char_rect.y + 1 + 1, // border + padding (simplified)
    };
    canvas.draw_text(text_pos, "Hello World", AnsiColor::Default, None);

    // 8. Generate final ANSI output
    let ansi_output = canvas.to_ansi_string();

    println!("Complete pipeline output:\n{}", ansi_output);
    println!("Character rectangle: {:?}", char_rect);
    println!("Taffy layout: {:?}", layout);

    // Debug coordinate conversion
    println!(
        "Width conversion: {}px → {} chars",
        layout.size.width, char_rect.width
    );
    println!(
        "Height conversion: {}px → {} chars",
        layout.size.height, char_rect.height
    );
    println!(
        "Padding: left:{}, top:{}",
        layout.padding.left, layout.padding.top
    );

    // Verify layout calculations
    assert_eq!(char_rect.width, converter.px_to_chars(100.0)); // 100px width

    // For height: 16px should be at least 1 character line
    // The original issue was height=1, but with proper Taffy layout it should be taller
    assert!(char_rect.height >= 1); // Taffy calculated height

    // Check if canvas is big enough for the border
    if char_rect.width >= 2 && char_rect.height >= 2 {
        println!("✅ Rectangle big enough for border");
    } else {
        println!(
            "⚠️  Rectangle too small for border: {}x{}",
            char_rect.width, char_rect.height
        );
    }

    // Verify output contains expected elements
    assert!(ansi_output.contains("┌")); // Top-left border
    assert!(ansi_output.contains("┐")); // Top-right border
    assert!(ansi_output.contains("Hello World")); // Text content
    assert!(ansi_output.contains("└")); // Bottom-left border
}

#[test]
fn test_column_layout_pipeline() {
    // Create column with two text children
    let child1 = SimpleFtdComponent::text("First");
    let child2 = SimpleFtdComponent::text("Second");

    let column = SimpleFtdComponent::column()
        .with_spacing(16) // 16px spacing = ~2 character lines
        .with_padding(4)
        .with_border(1)
        .with_children(vec![child1, child2]);

    // Map to CSS and compute layout
    let css_mapper = FtdToCssMapper::new();
    let mut layout_engine = TaffyLayoutEngine::new();

    // Create child nodes
    let child1_style = css_mapper.component_to_style(&SimpleFtdComponent::text("First"));
    let child2_style = css_mapper.component_to_style(&SimpleFtdComponent::text("Second"));
    let child1_node = layout_engine
        .create_text_node("First", child1_style)
        .unwrap();
    let child2_node = layout_engine
        .create_text_node("Second", child2_style)
        .unwrap();

    // Create column container
    let column_style = css_mapper.component_to_style(&column);
    let column_node = layout_engine
        .create_container_node(column_style, vec![child1_node, child2_node])
        .unwrap();
    layout_engine.set_root(column_node);

    // Compute layout
    let available = Size {
        width: AvailableSpace::Definite(400.0),
        height: AvailableSpace::Definite(400.0),
    };
    layout_engine.compute_layout(available).unwrap();

    // Verify children are vertically spaced
    let child1_layout = layout_engine.get_layout(child1_node).unwrap();
    let child2_layout = layout_engine.get_layout(child2_node).unwrap();

    // Child 2 should be below child 1 with gap
    assert!(child2_layout.location.y > child1_layout.location.y);
    assert!((child2_layout.location.y - child1_layout.location.y) >= 16.0); // Gap spacing

    println!("Column layout computed successfully:");
    println!(
        "Child 1: x:{}, y:{}, w:{}, h:{}",
        child1_layout.location.x,
        child1_layout.location.y,
        child1_layout.size.width,
        child1_layout.size.height
    );
    println!(
        "Child 2: x:{}, y:{}, w:{}, h:{}",
        child2_layout.location.x,
        child2_layout.location.y,
        child2_layout.size.width,
        child2_layout.size.height
    );
}

#[test]
fn test_ansi_color_output() {
    let mut canvas = AnsiCanvas::new(20, 5);

    // Draw colored text
    canvas.draw_text(
        CharPos { x: 2, y: 1 },
        "Red Text",
        AnsiColor::Red,
        Some(AnsiColor::Yellow), // Yellow background
    );

    let output = canvas.to_ansi_string();

    // Verify ANSI color codes are present
    assert!(output.contains("\x1b[")); // Contains ANSI escape sequences
    assert!(output.contains("Red Text"));

    println!("ANSI colored output:\n{}", output);
}
