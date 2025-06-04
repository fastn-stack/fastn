use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fastn-xtask", version, about = "fastn-xtask utility commands")]
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
            rt.block_on(fastn_xtask::template::run_template_command(&name)).unwrap();
        }
        None => {
            println!("Available commands: new <name>");
        }
    }
}
