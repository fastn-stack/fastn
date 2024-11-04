fn main() {
    let command = fastn::commands::parse();
    #[expect(unused_variables)]
    let config = fastn_core::Config::read(Box::new(fastn::DS::new()));
    // read config here and pass to everyone?
    // do common build stuff here
    match command {
        fastn::commands::Cli::Serve {
            port,
            watch,
            build,
            offline,
        } => fastn::commands::serve(port, watch, build, offline),
        fastn::commands::Cli::Render {
            path,
            key_values,
            action,
            output,
            browse,
            ui,
            offline,
        } => fastn::commands::render(path, key_values, action, output, browse, ui, offline),
        fastn::commands::Cli::Build { .. } => {}
        fastn::commands::Cli::Static { .. } => {}
        fastn::commands::Cli::Test { .. } => {}
        fastn::commands::Cli::Fmt(_) => {}
        fastn::commands::Cli::Lint(_) => {}
        fastn::commands::Cli::Upload { .. } => {}
        fastn::commands::Cli::Clone(_) => {}
    };
}
