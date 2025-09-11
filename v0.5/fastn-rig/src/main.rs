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
    /// Create a new account in the existing rig
    CreateAccount,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Determine fastn_home
    let fastn_home = fastn_rig::resolve_fastn_home(cli.home)
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

    match cli.command {
        Commands::Init => init_rig(fastn_home).await,
        Commands::Status => show_status(fastn_home).await,
        Commands::Entities => list_entities(fastn_home).await,
        Commands::SetCurrent { id52 } => set_current_entity(fastn_home, id52).await,
        Commands::SetOnline { id52, online } => set_entity_online(fastn_home, id52, online).await,
        Commands::Run => run_rig(fastn_home).await,
        Commands::CreateAccount => create_account(fastn_home).await,
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

async fn create_account(fastn_home: PathBuf) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!(
        "🔧 Creating new account in existing rig at {}",
        fastn_home.display()
    );

    // Create the accounts directory path
    let accounts_dir = fastn_home.join("accounts");

    if !accounts_dir.exists() {
        return Err(format!(
            "Accounts directory not found at {}. Initialize the rig first with 'fastn-rig init'.",
            accounts_dir.display()
        )
        .into());
    }

    // Create a new account directly using Account::create
    // This will generate a new ID52, create the account directory, and print the password
    let new_account = fastn_account::Account::create(&accounts_dir).await?;

    // Get the account ID52 from the newly created account
    let new_account_id52 = new_account
        .primary_id52()
        .await
        .ok_or("Failed to get primary ID52 from newly created account")?;

    println!("✅ Account created successfully in existing rig!");
    println!("👤 New Account ID52: {new_account_id52}");
    println!(
        "🏠 Account Directory: {}/accounts/{}",
        fastn_home.display(),
        new_account_id52
    );

    Ok(())
}
