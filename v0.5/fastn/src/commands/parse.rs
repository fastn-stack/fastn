// does error handling
pub fn parse() -> fastn::commands::Cli {
    if std::env::args().any(|x| x == "serve") {
        return fastn::commands::Cli::Serve(fastn::commands::Serve {
            listen: "127.0.0.1:8000".parse().unwrap(),
            watch: false,
            build: false,
            offline: false,
            protocol: "http".to_string(),
        });
    }

    // TODO
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
