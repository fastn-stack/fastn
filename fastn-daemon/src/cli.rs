#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "fastn")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Override the default FASTN_HOME directory
    #[arg(long, global = true, env = "FASTN_HOME")]
    pub fastn_home: Option<std::path::PathBuf>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Initialize fastn daemon (creates SSH folder in FASTN_HOME)
    Init,
    /// Run the fastn daemon service in foreground
    Daemon,
    /// Show daemon operational status and machine info
    Status,
    /// Connect to remote machines via SSH
    Ssh {
        /// Remote machine alias or id52
        target: String,
    },
}

pub async fn handle_cli(_cli: fastn_daemon::Cli) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement CLI handling
    Ok(())
}
