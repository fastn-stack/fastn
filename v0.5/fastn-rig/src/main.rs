use clap::{Parser, Subcommand};
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "fastn-rig")]
#[command(about = "A CLI for testing and managing fastn-rig")]
struct Cli {
    /// Path to fastn home directory
    #[arg(long, global = true)]
    home: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new rig
    Init,
    /// Show rig and entity status
    Status,
    /// List all entities and their online status
    Entities,
    /// Set the current entity
    SetCurrent {
        /// Entity ID52 to set as current
        id52: String,
    },
    /// Set entity online status
    SetOnline {
        /// Entity ID52
        id52: String,
        /// Online status (true/false)
        online: String,
    },
    /// Start the rig daemon
    Run,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Determine fastn_home
    let fastn_home = get_fastn_home(cli.home)?;

    match cli.command {
        Commands::Init => init_rig(fastn_home).await,
        Commands::Status => show_status(fastn_home).await,
        Commands::Entities => list_entities(fastn_home).await,
        Commands::SetCurrent { id52 } => set_current_entity(fastn_home, id52).await,
        Commands::SetOnline { id52, online } => set_entity_online(fastn_home, id52, online).await,
        Commands::Run => run_rig(fastn_home).await,
    }
}

fn get_fastn_home(home: Option<PathBuf>) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    match home {
        Some(path) => Ok(path),
        None => match std::env::var("FASTN_HOME") {
            Ok(env_path) => Ok(PathBuf::from(env_path)),
            Err(_) => {
                let proj_dirs = directories::ProjectDirs::from("com", "fastn", "fastn")
                    .ok_or("Failed to determine project directories")?;
                Ok(proj_dirs.data_dir().to_path_buf())
            }
        },
    }
}

async fn init_rig(fastn_home: PathBuf) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("🎉 Initializing new rig at {}", fastn_home.display());

    let (rig, _account_manager, primary_id52) = fastn_rig::Rig::create(fastn_home).await?;

    println!("✅ Rig initialized successfully!");
    println!("🔑 Rig ID52: {}", rig.id52());
    println!("👤 Owner: {}", rig.owner().id52());
    println!("📍 Primary account: {primary_id52}");

    Ok(())
}

async fn show_status(fastn_home: PathBuf) -> Result<(), Box<dyn Error + Send + Sync>> {
    let rig = fastn_rig::Rig::load(fastn_home)?;

    println!("📊 Rig Status");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔑 Rig ID52: {}", rig.id52());
    println!("👤 Owner: {}", rig.owner().id52());

    match rig.get_current().await {
        Ok(current) => println!("📍 Current entity: {current}"),
        Err(e) => println!("❌ Error getting current entity: {e}"),
    }

    Ok(())
}

async fn list_entities(fastn_home: PathBuf) -> Result<(), Box<dyn Error + Send + Sync>> {
    let rig = fastn_rig::Rig::load(fastn_home.clone())?;

    let account_manager = fastn_account::AccountManager::load(fastn_home).await?;

    println!("👥 Entities");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // List rig itself
    let rig_id52 = rig.id52();
    let rig_online = rig.is_entity_online(&rig_id52).await.unwrap_or(false);
    let status = if rig_online {
        "🟢 ONLINE"
    } else {
        "🔴 OFFLINE"
    };
    println!("⚙️  {rig_id52} (rig) - {status}");

    // List all accounts
    let all_endpoints = account_manager.get_all_endpoints().await?;
    for (id52, _secret_key, _account_path) in all_endpoints {
        let online = rig.is_entity_online(&id52).await.unwrap_or(false);
        let status = if online {
            "🟢 ONLINE"
        } else {
            "🔴 OFFLINE"
        };
        println!("👤 {id52} (account) - {status}");
    }

    Ok(())
}

async fn set_current_entity(
    fastn_home: PathBuf,
    id52: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let rig = fastn_rig::Rig::load(fastn_home)?;

    rig.set_current(&id52).await?;

    println!("✅ Set current entity to: {id52}");

    Ok(())
}

async fn set_entity_online(
    fastn_home: PathBuf,
    id52: String,
    online: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let rig = fastn_rig::Rig::load(fastn_home)?;

    let online_bool = match online.as_str() {
        "true" => true,
        "false" => false,
        _ => {
            eprintln!("Error: Online status must be 'true' or 'false'");
            std::process::exit(1);
        }
    };

    rig.set_entity_online(&id52, online_bool).await?;

    let status = if online_bool { "ONLINE" } else { "OFFLINE" };
    println!("✅ Set {id52} to {status}");

    Ok(())
}

async fn run_rig(fastn_home: PathBuf) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("🚀 Starting rig daemon...");
    fastn_rig::run(Some(fastn_home))
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}
