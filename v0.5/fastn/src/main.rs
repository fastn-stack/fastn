fn main() {
    #[expect(unused_variables)]
    let ds = fastn::DS::new();
    // read config here and pass to everyone?
    // do common build stuff here
    match fastn::commands::parse() {
        Ok(fastn::commands::Cli::Serve {
            port,
            watch,
            build,
            offline,
        }) => {
            fastn::commands::serve(port, watch, build, offline);
        }
        Ok(fastn::commands::Cli::Render { .. }) => {}
        Ok(fastn::commands::Cli::Build { .. }) => {}
        Ok(fastn::commands::Cli::Static { .. }) => {}
        Ok(fastn::commands::Cli::Test { .. }) => {}
        Ok(fastn::commands::Cli::Fmt(_)) => {}
        Ok(fastn::commands::Cli::Lint(_)) => {}
        Ok(fastn::commands::Cli::Upload { .. }) => {}
        Ok(fastn::commands::Cli::Clone(_)) => {}
        Err(_) => {}
    };
}
