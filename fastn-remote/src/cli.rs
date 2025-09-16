#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "fastn-remote")]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Start remote access listener for incoming connections
    Listen {
        /// Private key content (hex string)
        #[arg(long = "private-key", required = true)]
        private_key: String,

        /// Comma-separated list of allowed ID52s
        #[arg(long = "allowed", required = true)]
        allowed: String,
    },
    /// Interactive remote shell (PTY mode)
    Rshell {
        /// Private key content (hex string)
        #[arg(long = "private-key", required = true)]
        private_key: String,

        /// Target server ID52
        target: String,

        /// Optional command to execute (if not provided, starts interactive shell)
        command: Option<String>,
    },
    /// Execute command with separate stdout/stderr streams (automation mode)
    Rexec {
        /// Private key content (hex string)
        #[arg(long = "private-key", required = true)]
        private_key: String,

        /// Target server ID52
        target: String,

        /// Command to execute
        command: String,
    },
}

/// CLI wrapper for rshell command
pub async fn rshell_cli(private_key: &str, target: &str, command: Option<&str>) {
    use std::str::FromStr;

    let secret_key = match fastn_id52::SecretKey::from_str(private_key.trim()) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: Invalid private key format: {}", e);
            std::process::exit(1);
        }
    };

    let target_key = match fastn_id52::PublicKey::from_str(target.trim()) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: Invalid target ID52 '{}': {}", target, e);
            std::process::exit(1);
        }
    };

    fastn_remote::rshell(secret_key, target_key, command).await;
}

/// CLI wrapper for rexec command
pub async fn rexec_cli(private_key: &str, target: &str, command: &str) {
    use std::str::FromStr;

    let secret_key = match fastn_id52::SecretKey::from_str(private_key.trim()) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: Invalid private key format: {}", e);
            std::process::exit(1);
        }
    };

    let target_key = match fastn_id52::PublicKey::from_str(target.trim()) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: Invalid target ID52 '{}': {}", target, e);
            std::process::exit(1);
        }
    };

    fastn_remote::rexec(secret_key, target_key, command).await;
}

pub async fn handle_cli(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Listen {
            private_key,
            allowed,
        } => {
            fastn_remote::listen_cli(&private_key, &allowed).await;
        }
        Commands::Rshell {
            private_key,
            target,
            command,
        } => {
            fastn_remote::rshell_cli(&private_key, &target, command.as_deref()).await;
        }
        Commands::Rexec {
            private_key,
            target,
            command,
        } => {
            fastn_remote::rexec_cli(&private_key, &target, &command).await;
        }
    }

    Ok(())
}
