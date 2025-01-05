#[tokio::main]
async fn main() {
    let command = fastn::commands::parse();
    let mut section_provider = fastn::SectionProvider::default();
    let mut package = read(fastn_package::Package::reader(), &mut section_provider).await;
    let router = read(fastn_router::Router::reader(), &mut section_provider).await;
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

async fn read<T, C, P>(reader: fastn_continuation::Result<C>, provider: P) -> T
where
    C: fastn_continuation::Continuation<
        Output = fastn_utils::section_provider::PResult<T>,
        Needed = Vec<String>,
        Found = fastn_utils::section_provider::Found,
    >,
    P: fastn_continuation::AsyncMutProvider<Needed = C::Needed, Found = C::Found>,
{
    match reader.mut_consume_async(provider).await {
        Ok((value, warnings)) => {
            for warning in warnings {
                eprintln!("{warning:?}");
            }
            value
        }
        Err(diagnostics) => {
            eprintln!("failed to parse package: ");
            for diagnostic in diagnostics {
                eprintln!("{diagnostic:?}");
            }
            std::process::exit(1);
        }
    }
}
