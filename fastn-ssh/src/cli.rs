#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "fastn-ssh")]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Start SSH listener for incoming connections
    Listen {
        /// Private key content (hex string)
        #[arg(long = "private-key", required = true)]
        private_key: String,

        /// Comma-separated list of allowed ID52s
        #[arg(long = "allowed", required = true)]
        allowed: String,
    },
    /// Connect to remote SSH server
    Connect {
        /// Private key content (hex string)
        #[arg(long = "private-key", required = true)]
        private_key: String,

        /// Target server ID52
        target: String,
    },
}

pub async fn handle_cli(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Listen {
            private_key,
            allowed,
        } => {
            fastn_ssh::listen_cli(&private_key, &allowed).await;
        }
        Commands::Connect {
            private_key,
            target,
        } => {
            fastn_ssh::connect_cli(&private_key, &target).await;
        }
    }

    Ok(())
}
