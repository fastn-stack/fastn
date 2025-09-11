/// Embedded fastn document specifications for component browsing

/// Get embedded specification source by name
pub fn get_embedded_spec(spec_name: &str) -> Result<String, String> {
    let spec_path = spec_name.strip_suffix(".ftd").unwrap_or(spec_name);

    match spec_path {
        "text/basic" => Ok("-- ftd.text: Hello World".to_string()),
        "text/with-border" => Ok("-- ftd.text: Hello World\nborder-width.px: 1\npadding.px: 8\ncolor: red".to_string()),
        "components/button" => Ok("-- ftd.text: Click Me\nborder-width.px: 1\npadding.px: 4".to_string()),
        "forms/text-input" => Ok("-- ftd.text-input:\nplaceholder: Enter text here...\nborder-width.px: 1\npadding.px: 2".to_string()),
        "layout/column" => Ok("-- ftd.column:\nspacing.fixed.px: 16\n\n    -- ftd.text: Column 1\n    -- ftd.text: Column 2\n    -- ftd.text: Column 3\n\n-- end: ftd.column".to_string()),
        "layout/row" => Ok("-- ftd.row:\nspacing.fixed.px: 20\n\n    -- ftd.text: Item1\n    -- ftd.text: Item2\n    -- ftd.text: Item3\n\n-- end: ftd.row".to_string()),
        "forms/checkbox" => Ok("-- ftd.checkbox:\nchecked: false\n\n-- ftd.checkbox:\nchecked: true".to_string()),
        _ => Err(format!("Unknown specification: {}", spec_name))
    }
}

/// List all available embedded specifications
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

/// Get specifications organized by category
pub fn get_spec_categories() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("text", vec!["basic.ftd", "with-border.ftd"]),
        ("components", vec!["button.ftd"]),
        ("forms", vec!["text-input.ftd", "checkbox.ftd"]),
        ("layout", vec!["column.ftd", "row.ftd"]),
    ]
}
