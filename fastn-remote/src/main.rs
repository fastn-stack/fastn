#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: fastn_remote::Cli = clap::Parser::parse();
    fastn_remote::handle_cli(cli).await
}
