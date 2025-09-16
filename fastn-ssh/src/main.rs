#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: fastn_ssh::Cli = clap::Parser::parse();
    fastn_ssh::handle_cli(cli).await
}
