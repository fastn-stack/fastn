// does error handling
pub fn parse() -> fastn::commands::Cli {
    let args: Vec<String> = std::env::args().collect();
    parse_args(&args)
}

fn parse_args(args: &[String]) -> fastn::commands::Cli {
    // If only the program name is provided (no arguments), run the default behavior
    if args.len() == 1 {
        return fastn::commands::Cli::Run { home: None };
    }

    // Parse --home argument if present (can appear anywhere in args)
    let mut home = None;
    let mut filtered_args = vec![args[0].clone()]; // Keep program name
    let mut i = 1;
    while i < args.len() {
        if args[i].starts_with("--home=") {
            home = Some(std::path::PathBuf::from(&args[i][7..]));
            i += 1;
        } else if args[i] == "--home" && i + 1 < args.len() {
            home = Some(std::path::PathBuf::from(&args[i + 1]));
            i += 2;
        } else {
            filtered_args.push(args[i].clone());
            i += 1;
        }
    }

    // If no arguments left after filtering --home, run default behavior
    if filtered_args.len() == 1 {
        return fastn::commands::Cli::Run { home };
    }

    // Check for specific commands
    if filtered_args.iter().any(|x| x == "serve") {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_arguments() {
        let args = vec!["fastn".to_string()];
        match parse_args(&args) {
            fastn::commands::Cli::Run { home } => {
                assert_eq!(home, None);
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_only_home_argument_with_equals() {
        let args = vec!["fastn".to_string(), "--home=/custom/path".to_string()];
        match parse_args(&args) {
            fastn::commands::Cli::Run { home } => {
                assert_eq!(home, Some(std::path::PathBuf::from("/custom/path")));
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_only_home_argument_separate() {
        let args = vec![
            "fastn".to_string(),
            "--home".to_string(),
            "/custom/path".to_string(),
        ];
        match parse_args(&args) {
            fastn::commands::Cli::Run { home } => {
                assert_eq!(home, Some(std::path::PathBuf::from("/custom/path")));
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_serve_command() {
        let args = vec!["fastn".to_string(), "serve".to_string()];
        match parse_args(&args) {
            fastn::commands::Cli::Serve(serve) => {
                assert_eq!(serve.listen.to_string(), "127.0.0.1:8000");
                assert!(!serve.watch);
                assert!(!serve.build);
                assert!(!serve.offline);
                assert_eq!(serve.protocol, "http");
            }
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_serve_with_home() {
        let args = vec![
            "fastn".to_string(),
            "--home=/custom/path".to_string(),
            "serve".to_string(),
        ];
        match parse_args(&args) {
            fastn::commands::Cli::Serve(_) => {
                // Home is ignored for serve command currently
            }
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_home_after_serve() {
        let args = vec![
            "fastn".to_string(),
            "serve".to_string(),
            "--home=/custom/path".to_string(),
        ];
        match parse_args(&args) {
            fastn::commands::Cli::Serve(_) => {
                // Home is ignored for serve command currently
            }
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_render_fallback() {
        let args = vec!["fastn".to_string(), "unknown".to_string()];
        match parse_args(&args) {
            fastn::commands::Cli::Render(render) => {
                assert_eq!(render.path, "/");
                assert!(render.key_values.is_empty());
                assert!(!render.browse);
                assert!(!render.offline);
                assert!(!render.strict);
            }
            _ => panic!("Expected Render command as fallback"),
        }
    }

    #[test]
    fn test_home_with_spaces() {
        let args = vec![
            "fastn".to_string(),
            "--home=/path with spaces/dir".to_string(),
        ];
        match parse_args(&args) {
            fastn::commands::Cli::Run { home } => {
                assert_eq!(
                    home,
                    Some(std::path::PathBuf::from("/path with spaces/dir"))
                );
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_home_argument_without_value() {
        // Test that --home without a value is treated as a regular argument
        let args = vec!["fastn".to_string(), "--home".to_string()];
        match parse_args(&args) {
            fastn::commands::Cli::Render(_) => {
                // Falls back to render since --home without value is not consumed
            }
            _ => panic!("Expected Render command as fallback"),
        }
    }

    #[test]
    fn test_multiple_home_arguments() {
        // Last --home should win
        let args = vec![
            "fastn".to_string(),
            "--home=/first".to_string(),
            "--home=/second".to_string(),
        ];
        match parse_args(&args) {
            fastn::commands::Cli::Run { home } => {
                assert_eq!(home, Some(std::path::PathBuf::from("/second")));
            }
            _ => panic!("Expected Run command"),
        }
    }
}
