// does error handling
pub fn parse() -> fastn::commands::Cli {
    // TODO
    fastn::commands::Cli::Render(fastn::commands::Render {
        path: "/".to_string(),
        key_values: vec![],
        action: fastn::Action::Read,
        output: None,
        browse: false,
        ui: fastn::commands::UI::Terminal,
        offline: false,
    })
}
