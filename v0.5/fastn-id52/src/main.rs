struct Cli {
    command: Command,
}

enum Command {
    Generate(GenerateOptions),
    #[cfg(feature = "dns")]
    Resolve(ResolveOptions),
    Help,
}

struct GenerateOptions {
    storage: StorageMethod,
    short_output: bool,
}

enum StorageMethod {
    Keyring,
    File(String),
    Stdout,
}

#[cfg(feature = "dns")]
struct ResolveOptions {
    domain: String,
    scope: String,
}

impl Cli {
    fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();

        if args.len() < 2 {
            return Cli {
                command: Command::Help,
            };
        }

        match args[1].as_str() {
            "generate" => {
                let options = Self::parse_generate_options(&args[2..]);
                Cli {
                    command: Command::Generate(options),
                }
            }
            #[cfg(feature = "dns")]
            "resolve" => {
                let options = Self::parse_resolve_options(&args[2..]);
                Cli {
                    command: Command::Resolve(options),
                }
            }
            "help" | "--help" | "-h" => Cli {
                command: Command::Help,
            },
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    fn parse_generate_options(args: &[String]) -> GenerateOptions {
        let mut storage = StorageMethod::Keyring;
        let mut short_output = false;
        let mut explicit_keyring = false;
        let mut i = 0;

        while i < args.len() {
            match args[i].as_str() {
                "-k" | "--keyring" => {
                    explicit_keyring = true;
                    storage = StorageMethod::Keyring;
                    i += 1;
                }
                "-f" | "--file" => {
                    if explicit_keyring {
                        eprintln!("Error: Cannot use both --keyring and --file options together");
                        std::process::exit(1);
                    }

                    // Check if next arg exists and is not a flag
                    if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                        let filename = args[i + 1].clone();
                        storage = if filename == "-" {
                            StorageMethod::Stdout
                        } else {
                            StorageMethod::File(filename)
                        };
                        i += 2;
                    } else {
                        // Flag present but no value, use default
                        storage = StorageMethod::File(".fastn.secret-key".to_string());
                        i += 1;
                    }
                }
                "-s" | "--short" => {
                    short_output = true;
                    i += 1;
                }
                _ => {
                    eprintln!("Unknown option for generate: {}", args[i]);
                    eprintln!();
                    eprintln!(
                        "Usage: fastn-id52 generate [-k|--keyring] [-f|--file [FILENAME]] [-s|--short]"
                    );
                    std::process::exit(1);
                }
            }
        }

        GenerateOptions {
            storage,
            short_output,
        }
    }

    #[cfg(feature = "dns")]
    fn parse_resolve_options(args: &[String]) -> ResolveOptions {
        if args.len() != 2 {
            eprintln!("Error: resolve command requires exactly 2 arguments: <domain> <scope>");
            eprintln!();
            eprintln!("Usage: fastn-id52 resolve <domain> <scope>");
            eprintln!("Example: fastn-id52 resolve fifthtry.com malai");
            std::process::exit(1);
        }

        ResolveOptions {
            domain: args[0].clone(),
            scope: args[1].clone(),
        }
    }

    #[cfg(feature = "dns")]
    async fn run(self) {
        match self.command {
            Command::Help => {
                print_help();
            }
            Command::Generate(options) => {
                handle_generate(options);
            }
            Command::Resolve(options) => {
                handle_resolve(options).await;
            }
        }
    }

    #[cfg(not(feature = "dns"))]
    fn run(self) {
        match self.command {
            Command::Help => {
                print_help();
            }
            Command::Generate(options) => {
                handle_generate(options);
            }
        }
    }
}

#[cfg(feature = "dns")]
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    cli.run().await;
}

#[cfg(not(feature = "dns"))]
fn main() {
    let cli = Cli::parse();
    cli.run();
}

fn print_help() {
    eprintln!("fastn-id52 - Entity identity generation and DNS resolution for fastn peer-to-peer network");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  fastn-id52 <COMMAND>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  generate    Generate a new entity identity");
    #[cfg(feature = "dns")]
    eprintln!("  resolve     Resolve a public key from DNS TXT records");
    eprintln!("  help        Print this help message");
    eprintln!();
    eprintln!("Generate command options:");
    eprintln!("  -k, --keyring           Store in system keyring (default behavior)");
    eprintln!("  -f, --file [FILENAME]   Save to file (use '-' for stdout)");
    eprintln!("  -s, --short             Only print ID52, no descriptive messages");
    eprintln!();
    #[cfg(feature = "dns")]
    {
        eprintln!("Resolve command usage:");
        eprintln!("  fastn-id52 resolve <domain> <scope>");
        eprintln!();
        eprintln!("  Looks for DNS TXT records in format: <scope>=<id52>");
        eprintln!("  Example: fastn-id52 resolve fifthtry.com malai");
        eprintln!("  This looks for TXT record: \"malai=<52-char-public-key>\"");
        eprintln!();
    }
    eprintln!("By default, the secret key is stored in the system keyring and only the");
    eprintln!("public key (ID52) is printed. Use -f to override this behavior.");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  fastn-id52 generate              # Store in keyring, print ID52");
    eprintln!("  fastn-id52 generate -s           # Store in keyring, only ID52 on stderr");
    eprintln!("  fastn-id52 generate -f -         # Print secret to stdout, ID52 to stderr");
    eprintln!("  fastn-id52 generate -f - -s      # Print secret to stdout, only ID52 on stderr");
    #[cfg(feature = "dns")]
    eprintln!("  fastn-id52 resolve example.com alice  # Resolve public key for alice@example.com");
}

fn handle_generate(options: GenerateOptions) {
    // Generate new key
    let secret_key = fastn_id52::SecretKey::generate();
    let id52 = secret_key.id52();

    // Handle output based on selected method
    match options.storage {
        StorageMethod::Stdout => {
            // Output secret to stdout
            println!("{secret_key}");
            if options.short_output {
                eprintln!("{id52}");
            } else {
                eprintln!("Public Key (ID52): {id52}");
            }
        }
        StorageMethod::File(ref filename) => {
            // Save to file
            save_to_file(filename, &secret_key);
            if options.short_output {
                eprintln!("{id52}");
            } else {
                eprintln!("Private key saved to `{filename}`.");
                eprintln!("WARNING: File storage is less secure than keyring storage.");
                eprintln!("Public Key (ID52): {id52}");
            }
        }
        StorageMethod::Keyring => {
            // Store in keyring
            save_to_keyring(&secret_key, options.short_output);
            // Print the public key
            if options.short_output {
                eprintln!("{id52}");
            } else {
                println!("{id52}");
            }
        }
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
}

fn save_to_keyring(secret_key: &fastn_id52::SecretKey, short_output: bool) {
    let id52 = secret_key.id52();

    match secret_key.store_in_keyring() {
        Ok(_) => {
            if !short_output {
                eprintln!("Secret key stored securely in system keyring");
                eprintln!("You can view it in your password manager under:");
                eprintln!("  Service: fastn");
                eprintln!("  Account: {id52}");
            }
        }
        Err(e) => {
            eprintln!("ERROR: Failed to store secret key in keyring: {e}");
            if !short_output {
                eprintln!();
                eprintln!("The system keyring is not accessible. To proceed, you must");
                eprintln!("explicitly choose an alternative:");
                eprintln!("  - Use --file to save the secret key to a file (WARNING: less secure)");
                eprintln!("  - Use --file - to output the key to stdout");
                eprintln!();
                eprintln!(
                    "Never store secret keys in plain text files unless absolutely necessary."
                );
            }
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "dns")]
async fn handle_resolve(options: ResolveOptions) {
    use fastn_id52::PublicKey;
    
    println!("Resolving public key for scope '{}' on domain '{}'...", options.scope, options.domain);
    
    match PublicKey::resolve(&options.domain, &options.scope).await {
        Ok(public_key) => {
            println!();
            println!("✓ Success! Public key found:");
            println!("{}", public_key.id52());
        }
        Err(e) => {
            println!();
            println!("✗ Failed to resolve public key:");
            println!("{}", e);
            println!();
            println!("How to fix this:");
            println!("1. Make sure the domain '{}' has a DNS TXT record", options.domain);
            println!("2. The TXT record should be in format: \"{}=<52-character-public-key>\"", options.scope);
            println!("3. Example TXT record: \"{}=i66fo538lfl5ombdf6tcdbrabp4hmp9asv7nrffuc2im13ct4q60\"", options.scope);
            println!();
            println!("To add a TXT record:");
            println!("• If using a DNS provider (Cloudflare, Route53, etc.):");
            println!("  - Add a new TXT record for domain '{}'", options.domain);
            println!("  - Set the value to: {}=<your-public-key-id52>", options.scope);
            println!("• If managing DNS yourself:");
            println!("  - Add to your zone file: {} IN TXT \"{}=<your-public-key-id52>\"", options.domain, options.scope);
            println!();
            println!("Note: DNS changes can take a few minutes to propagate.");
            
            std::process::exit(1);
        }
    }
}
