#[tokio::main]
async fn main() {
    fastn_observer::observe();
    let command = fastn::commands::parse();
    let mut section_provider = fastn::SectionProvider::default();
    let module = fastn_section::Module::main(&mut section_provider.arena);
    let mut package = section_provider.read(fastn_package::reader(module)).await;
    let router = section_provider.read(fastn_router::reader()).await;
    // read config here and pass to everyone?
    // do common build stuff here
    match command {
        fastn::commands::Cli::Serve(input) => input.run(package, router).await,
        fastn::commands::Cli::Render(input) => input.run(&mut package, router).await,
        fastn::commands::Cli::Build(input) => input.run(package).await,
        fastn::commands::Cli::Static { .. } => {}
        fastn::commands::Cli::Test { .. } => {}
        fastn::commands::Cli::Fmt(_) => {}
        fastn::commands::Cli::Upload { .. } => {}
        fastn::commands::Cli::Clone(_) => {}
    };
}
