use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Main function attribute macro that eliminates boilerplate for fastn applications
///
/// # Example
///
/// ```rust,ignore
/// #[fastn_p2p::main]
/// async fn main() -> eyre::Result<()> {
///     fastn_p2p::spawn(my_service());
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn main(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // For now, just implement the basic case - TODO: parse configuration
    let config = MacroConfig::default();
    
    // Generate the expanded main function
    expand_main(config, input_fn).into()
}

#[derive(Debug, Default)]
struct MacroConfig {
    logging: LoggingConfig,
    shutdown_mode: ShutdownMode,
    shutdown_timeout: String,
    double_ctrl_c_window: String,
    status_fn: Option<syn::Path>,
}

#[derive(Debug)]
enum LoggingConfig {
    Enabled(bool),
    Level(String),
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig::Enabled(true)
    }
}

#[derive(Debug)]
enum ShutdownMode {
    SingleCtrlC,
    DoubleCtrlC,
}

impl Default for ShutdownMode {
    fn default() -> Self {
        ShutdownMode::SingleCtrlC
    }
}

// TODO: Implement configuration parsing for syn 2.0 API

fn expand_main(config: MacroConfig, input_fn: ItemFn) -> proc_macro2::TokenStream {
    let user_fn_name = syn::Ident::new("__fastn_user_main", proc_macro2::Span::call_site());
    let fn_block = &input_fn.block;
    let fn_attrs = &input_fn.attrs;
    let fn_vis = &input_fn.vis;
    
    // Generate logging setup code
    let logging_setup = generate_logging_setup(&config.logging);
    
    // Generate shutdown handling code
    let shutdown_setup = generate_shutdown_setup(&config);
    
    quote! {
        #(#fn_attrs)*
        #fn_vis fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
            // Initialize tokio runtime
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?
                .block_on(async {
                    #logging_setup
                    
                    // Initialize global graceful singleton (automatically done via LazyLock)
                    
                    #shutdown_setup
                    
                    // Call user's main function
                    let result = #user_fn_name().await;
                    
                    // TODO: Trigger graceful shutdown
                    // fastn_p2p::coordination::trigger_shutdown().await?;
                    
                    result
                })
        }
        
        async fn #user_fn_name() -> std::result::Result<(), Box<dyn std::error::Error>> #fn_block
    }
}

fn generate_logging_setup(logging: &LoggingConfig) -> proc_macro2::TokenStream {
    match logging {
        LoggingConfig::Enabled(false) => quote! {
            // Logging disabled
        },
        LoggingConfig::Enabled(true) => quote! {
            // Simple logging setup with RUST_LOG override
            let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .init();
        },
        LoggingConfig::Level(_level) => {
            quote! {
                // TODO: Logging setup with custom level and RUST_LOG override
            }
        }
    }
}

fn generate_shutdown_setup(config: &MacroConfig) -> proc_macro2::TokenStream {
    match config.shutdown_mode {
        ShutdownMode::SingleCtrlC => {
            let _timeout = &config.shutdown_timeout;
            quote! {
                // TODO: Single Ctrl+C mode - immediate graceful shutdown
            }
        }
        ShutdownMode::DoubleCtrlC => {
            quote! {
                // TODO: Double Ctrl+C mode - status on first, shutdown on second
            }
        }
    }
}