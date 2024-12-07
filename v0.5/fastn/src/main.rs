#[tokio::main]
async fn main() {
    let command = fastn::commands::parse();
    let mut package = fastn_package::Package::reader().consume(fastn::full_filler);
    let router = fastn_router::Router::reader()
        .consume_async(fastn::full_filler_async)
        .await;
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
