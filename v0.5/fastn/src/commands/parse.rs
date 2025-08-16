// does error handling
pub fn parse() -> fastn::commands::Cli {
    let args: Vec<String> = std::env::args().collect();

    // If only the program name is provided (no arguments), run the default behavior
    if args.len() == 1 {
        return fastn::commands::Cli::Run;
    }

    // Check for specific commands
    if args.iter().any(|x| x == "serve") {
        return fastn::commands::Cli::Serve(fastn::commands::Serve {
            listen: "127.0.0.1:8000".parse().unwrap(),
            watch: false,
            build: false,
            offline: false,
            protocol: "http".to_string(),
        });
    }

    // TODO: Parse other commands properly
    // For now, default to Render for backward compatibility
    fastn::commands::Cli::Render(fastn::commands::Render {
        path: "/".to_string(),
        key_values: vec![],
        action: fastn::Action::Read,
        output: None,
        browse: false,
        ui: fastn::commands::UI::Terminal,
        offline: false,
        strict: false,
    })
}
