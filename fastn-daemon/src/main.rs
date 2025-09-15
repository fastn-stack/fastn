#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: fastn_daemon::Cli = clap::Parser::parse();
    fastn_daemon::handle_cli(cli).await
}
