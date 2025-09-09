use fastn_ascii_renderer::TaffyLayoutEngine;
use taffy::{Style, Size, AvailableSpace, Dimension};

#[test]
fn test_basic_taffy_integration() {
    let mut layout_engine = TaffyLayoutEngine::new();
    
    // Create a simple text node
    let style = Style {
        size: Size {
            width: Dimension::Length(100.0.into()),
            height: Dimension::Length(20.0.into()),
        },
        ..Default::default()
    };
    
    let node = layout_engine.create_text_node("Hello World", style).unwrap();
    layout_engine.set_root(node);
    
    // Compute layout
    let available = Size {
        width: AvailableSpace::Definite(800.0),
        height: AvailableSpace::Definite(600.0),
    };
    
    layout_engine.compute_layout(available).unwrap();
    
    // Verify layout was computed
    let layout = layout_engine.get_layout(node).unwrap();
    assert_eq!(layout.size.width, 100.0);
    assert_eq!(layout.size.height, 20.0);
}

#[test]
fn test_column_layout() {
    let mut layout_engine = TaffyLayoutEngine::new();
    
    // Create two text children
    let child1_style = Style {
        size: Size {
            width: Dimension::Length(50.0.into()),
            height: Dimension::Length(16.0.into()),
        },
        ..Default::default()
    };
    
    let child2_style = Style {
        size: Size {
            width: Dimension::Length(75.0.into()),
            height: Dimension::Length(16.0.into()),
        },
        ..Default::default()
    };
    
    let child1 = layout_engine.create_text_node("Child 1", child1_style).unwrap();
    let child2 = layout_engine.create_text_node("Child 2", child2_style).unwrap();
    
    // Create column container
    let column_style = Style {
        flex_direction: taffy::FlexDirection::Column,
        gap: Size {
            width: taffy::LengthPercentage::Length(8.0.into()),
            height: taffy::LengthPercentage::Length(8.0.into()),
        },
        ..Default::default()
    };
    
    let column = layout_engine.create_container_node(column_style, vec![child1, child2]).unwrap();
    layout_engine.set_root(column);
    
    // Compute layout
    let available = Size {
        width: AvailableSpace::Definite(200.0),
        height: AvailableSpace::Definite(200.0),
    };
    
    layout_engine.compute_layout(available).unwrap();
    
    // Verify column layout
    let column_layout = layout_engine.get_layout(column).unwrap();
    let child1_layout = layout_engine.get_layout(child1).unwrap();
    let child2_layout = layout_engine.get_layout(child2).unwrap();
    
    // Child 1 should be at top
    assert_eq!(child1_layout.location.y, 0.0);
    
    // Child 2 should be below child 1 + gap
    assert_eq!(child2_layout.location.y, 16.0 + 8.0); // height + gap
    
    // Column should be tall enough for both children + gap
    assert_eq!(column_layout.size.height, 16.0 + 8.0 + 16.0); // child1 + gap + child2
}

#[test]  
fn test_debug_output() {
    let mut layout_engine = TaffyLayoutEngine::new();
    
    let style = Style {
        size: Size {
            width: Dimension::Length(80.0.into()),
            height: Dimension::Length(24.0.into()),
        },
        ..Default::default()
    };
    
    let node = layout_engine.create_text_node("Debug Test", style).unwrap();
    layout_engine.set_root(node);
    
    let available = Size {
        width: AvailableSpace::Definite(400.0),
        height: AvailableSpace::Definite(300.0),
    };
    
    layout_engine.compute_layout(available).unwrap();
    
    // Test debug output generation
    let debug_layouts = layout_engine.debug_layouts();
    assert_eq!(debug_layouts.len(), 1);
    assert_eq!(debug_layouts[0].1.size.width, 80.0);
    assert_eq!(debug_layouts[0].1.size.height, 24.0);
}