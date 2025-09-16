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
    /// Execute single command on remote SSH server
    Exec {
        /// Private key content (hex string)
        #[arg(long = "private-key", required = true)]
        private_key: String,

        /// Target server ID52
        target: String,

        /// Command to execute
        command: String,
    },
    /// Start interactive TTY session on remote SSH server
    Tty {
        /// Private key content (hex string)
        #[arg(long = "private-key", required = true)]
        private_key: String,

        /// Target server ID52
        target: String,
    },
    /// Start stream-based session (separate stdout/stderr)
    Spawn {
        /// Private key content (hex string)
        #[arg(long = "private-key", required = true)]
        private_key: String,

        /// Target server ID52
        target: String,

        /// Command to execute with streaming
        command: String,
    },
}

pub async fn handle_cli(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Listen {
            private_key,
            allowed,
        } => {
            fastn_remote::listen_cli(&private_key, &allowed).await;
        }
        Commands::Exec {
            private_key,
            target,
            command,
        } => {
            fastn_remote::exec_cli(&private_key, &target, &command).await;
        }
        Commands::Tty {
            private_key,
            target,
        } => {
            fastn_remote::tty_cli(&private_key, &target).await;
        }
        Commands::Spawn {
            private_key,
            target,
            command,
        } => {
            fastn_remote::spawn_cli(&private_key, &target, &command).await;
        }
    }

    Ok(())
}
