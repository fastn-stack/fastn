fn main() {
    let command = fastn::commands::parse();
    let config = fastn_core::Config::read(Box::new(fastn::DS::new()));
    // read config here and pass to everyone?
    // do common build stuff here
    match command {
        fastn::commands::Cli::Serve(input) => fastn::commands::serve(config, input),
        fastn::commands::Cli::Render(input) => fastn::commands::render(config, input),
        fastn::commands::Cli::Build { .. } => {}
        fastn::commands::Cli::Static { .. } => {}
        fastn::commands::Cli::Test { .. } => {}
        fastn::commands::Cli::Fmt(_) => {}
        fastn::commands::Cli::Lint(_) => {}
        fastn::commands::Cli::Upload { .. } => {}
        fastn::commands::Cli::Clone(_) => {}
    };
}
