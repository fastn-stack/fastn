use crate::{SimpleFtdComponent, FtdSize, ComponentType};

/// Embedded component specifications - should be compile-time included
pub fn get_embedded_spec(spec_name: &str) -> Result<SimpleFtdComponent, String> {
    let component_path = spec_name.strip_suffix(".ftd").unwrap_or(spec_name);
    
    match component_path {
        "text/basic" => {
            Ok(SimpleFtdComponent::text("Hello World"))
        },
        "text/with-border" => {
            Ok(SimpleFtdComponent::text("Hello World")
                .with_border(1)         // border-width.px: 1
                .with_padding(8))       // padding.px: 8  
        },
        "components/button" => {
            Ok(SimpleFtdComponent::text("Click Me")
                .with_border(1)         // border-width.px: 1
                .with_padding(4))       // padding.px: 4
        },
        "forms/text-input" => {
            Ok(SimpleFtdComponent::text("Enter text here...")
                .with_border(1)         // border-width.px: 1
                .with_padding(2)        // padding.px: 2
                .with_width(FtdSize::FillContainer)) // width: fill-container
        },
        "layout/column" => {
            Ok(SimpleFtdComponent::column()
                .with_spacing(16)       // spacing.fixed.px: 16
                .with_children(vec![
                    SimpleFtdComponent::text("Column 1"),
                    SimpleFtdComponent::text("Column 2"),
                    SimpleFtdComponent::text("Column 3"),
                ]))
        },
        "layout/row" => {
            Ok(SimpleFtdComponent::text("Item1    Item2    Item3"))
        },
        "forms/checkbox" => {
            Ok(SimpleFtdComponent::text("☐ Unchecked\n☑ Checked"))
        },
        _ => Err(format!("Unknown component spec: {}", spec_name))
    }
}

/// List all available embedded specs
pub fn list_embedded_specs() -> Vec<&'static str> {
    vec![
        "text/basic.ftd",
        "text/with-border.ftd", 
        "components/button.ftd",
        "forms/text-input.ftd",
        "layout/column.ftd",
        "layout/row.ftd",
        "forms/checkbox.ftd",
    ]
}