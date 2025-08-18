mod cli;

fn main() {
    let cli: cli::Cli = clap::Parser::parse();

    if let Err(e) = cli::run_command(cli) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
