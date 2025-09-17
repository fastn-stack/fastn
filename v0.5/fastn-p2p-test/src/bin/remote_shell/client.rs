//! Minimal Remote Shell Client
//! 
//! Execute commands on remote machines over P2P.
//! 
//! Usage: remote-shell-client <target_id52> <command...>

use clap::Parser;
use fastn_p2p_test::{
    protocols::{parse_id52, get_or_generate_key},
    remote_shell::{RemoteShellProtocol, ExecuteRequest, ExecuteResponse, RemoteShellError},
};

#[derive(Parser)]
#[command(about = "Execute commands on remote machines over P2P")]
struct Args {
    /// Target machine ID52
    target: String,
    
    /// Private key (optional)
    #[arg(long)]
    private_key: Option<String>,
    
    /// Command to execute
    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}

#[fastn_context::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    
    // Parse inputs
    let target = parse_id52(&args.target)?;
    let private_key = get_or_generate_key(args.private_key.as_deref())?;
    let command = if args.command.is_empty() {
        vec!["whoami".to_string()]
    } else {
        args.command
    };
    
    // Execute remote command - this is the core fastn-p2p usage!
    let request = ExecuteRequest { command };
    let result: Result<ExecuteResponse, RemoteShellError> = fastn_p2p::client::call(
        private_key,
        target,
        RemoteShellProtocol::Execute,
        request,
    ).await?;
    
    // Handle response
    match result {
        Ok(response) => {
            print!("{}", response.stdout);
            eprint!("{}", response.stderr);
            std::process::exit(response.exit_code);
        }
        Err(error) => {
            eprintln!("Remote error: {}", error);
            std::process::exit(1);
        }
    }
}