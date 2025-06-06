use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask", version, about = "fastn xtask utility commands")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    New {
        #[arg(required = true, help = "Name of the app to create")]
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::New { name }) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(e) = rt.block_on(fastn_xtask::core::new_app(&name)) {
                eprintln!("Error creating new app: {:?}", e);
                std::process::exit(1);
            }
        }
        None => {
            println!("Available commands:");
            println!("  new <name>         - Create a new fastn app");
        }
    }
}
