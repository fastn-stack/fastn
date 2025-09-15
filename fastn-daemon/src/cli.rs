#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "fastn-daemon")]
#[command(arg_required_else_help = true)]
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
    Run,
    /// Show daemon operational status and machine info
    Status,
    /// Connect to remote machines via SSH
    Ssh {
        /// Remote machine alias or id52
        target: String,
    },
}

pub async fn handle_cli(cli: fastn_daemon::Cli) -> Result<(), Box<dyn std::error::Error>> {
    let fastn_home = cli.fastn_home.unwrap_or_else(|| {
        dirs::data_dir()
            .expect("Failed to get data directory")
            .join("fastn")
    });

    println!("Using FASTN_HOME: {fastn_home:?}");

    match cli.command {
        Commands::Init => fastn_daemon::init(&fastn_home).await,
        Commands::Run => fastn_daemon::run(&fastn_home).await,
        Commands::Status => fastn_daemon::status(&fastn_home).await,
        Commands::Ssh { target } => fastn_daemon::ssh(&fastn_home, &target).await,
    };

    Ok(())
}

pub fn add_subcommands(app: clap::Command) -> clap::Command {
    app.subcommand(
        clap::Command::new("init")
            .about("Initialize fastn daemon (creates SSH folder in FASTN_HOME)"),
    )
    .subcommand(clap::Command::new("daemon").about("Run the fastn daemon service in foreground"))
    .subcommand(
        clap::Command::new("status").about("Show daemon operational status and machine info"),
    )
    .subcommand(
        clap::Command::new("ssh")
            .about("Connect to remote machines via SSH")
            .arg(clap::arg!(target: <TARGET> "Remote machine alias or id52").required(true)),
    )
    .arg(
        clap::arg!(--"fastn-home" <FASTN_HOME> "Override the default FASTN_HOME directory")
            .global(true),
    )
}

pub async fn handle_daemon_commands(
    matches: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    if matches.subcommand_matches("init").is_some()
        || matches.subcommand_matches("daemon").is_some()
        || matches.subcommand_matches("status").is_some()
        || matches.subcommand_matches("ssh").is_some()
    {
        let fastn_home = matches.get_one::<std::path::PathBuf>("fastn-home").cloned();

        let command = if matches.subcommand_matches("init").is_some() {
            Commands::Init
        } else if matches.subcommand_matches("daemon").is_some() {
            Commands::Run
        } else if matches.subcommand_matches("status").is_some() {
            Commands::Status
        } else if let Some(ssh_matches) = matches.subcommand_matches("ssh") {
            let target = ssh_matches.get_one::<String>("target").unwrap().clone();
            Commands::Ssh { target }
        } else {
            return Ok(());
        };

        let cli = Cli {
            command,
            fastn_home,
        };

        handle_cli(cli).await?;
    }

    Ok(())
}
