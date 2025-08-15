fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_help();
        std::process::exit(0);
    }

    match args[1].as_str() {
        "generate" => handle_generate(&args[2..]),
        "help" | "--help" | "-h" => {
            print_help();
            std::process::exit(0);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    eprintln!("fastn-id52 - Entity identity management for fastn peer-to-peer network");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  fastn-id52 <COMMAND>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  generate    Generate a new entity identity");
    eprintln!("  help        Print this help message");
    eprintln!();
    eprintln!("Generate command options:");
    eprintln!(
        "  -f, --file [FILENAME]   Save to file (default: .fastn.secret-key if flag present without value)"
    );
    eprintln!("  -p, --print             Print to stdout (requires explicit flag for safety)");
}

fn handle_generate(args: &[String]) {
    let mut filename: Option<String> = None;
    let mut print_to_stdout = false;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--file" => {
                // Check if next arg exists and is not a flag
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    filename = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    // Flag present but no value, use default
                    filename = Some(".fastn.secret-key".to_string());
                    i += 1;
                }
            }
            "-p" | "--print" => {
                print_to_stdout = true;
                i += 1;
            }
            _ => {
                eprintln!("Unknown option for generate: {}", args[i]);
                eprintln!();
                eprintln!("Usage: fastn-id52 generate [-f|--file [FILENAME]] [-p|--print]");
                std::process::exit(1);
            }
        }
    }

    // Check that exactly one output method is specified
    if filename.is_some() && print_to_stdout {
        eprintln!("Error: Cannot use both --file and --print options");
        std::process::exit(1);
    }

    if filename.is_none() && !print_to_stdout {
        eprintln!("Error: Must specify either --file or --print option");
        eprintln!();
        eprintln!("Usage: fastn-id52 generate [-f|--file [FILENAME]] [-p|--print]");
        std::process::exit(1);
    }

    // Generate new key
    let secret_key = fastn_id52::SecretKey::generate();
    let id52 = secret_key.id52();

    eprintln!("Generated Public Key (ID52): {id52}");

    if print_to_stdout {
        // Use Display implementation which outputs hex
        println!("{secret_key}");
    } else if let Some(ref filename) = filename {
        save_to_file(filename, &secret_key);
    }
}

fn save_to_file(filename: &str, secret_key: &fastn_id52::SecretKey) {
    use std::io::Write;

    if std::path::Path::new(filename).exists() {
        eprintln!("File `{filename}` already exists. Please choose a different file name.");
        std::process::exit(1);
    }

    let mut file = match std::fs::File::create(filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create file `{filename}`: {e}");
            std::process::exit(1);
        }
    };

    // Use Display implementation which outputs hex
    match writeln!(file, "{secret_key}") {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to write secret key to file `{filename}`: {e}");
            std::process::exit(1);
        }
    }

    eprintln!("Private key saved to `{filename}`.");
}
