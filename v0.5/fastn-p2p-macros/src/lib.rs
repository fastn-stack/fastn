use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, ExprLit, ItemFn, Lit, Meta, MetaNameValue, Token, parse::Parse, parse::ParseStream,
    parse_macro_input,
};

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
pub fn main(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    // Parse configuration from attributes
    let config = if args.is_empty() {
        MacroConfig::default()
    } else {
        match syn::parse::<MacroArgs>(args) {
            Ok(args) => args.into_config(),
            Err(err) => return err.to_compile_error().into(),
        }
    };

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

/// Parse macro arguments in syn 2.0 style
struct MacroArgs {
    args: syn::punctuated::Punctuated<Meta, Token![,]>,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MacroArgs {
            args: input.parse_terminated(Meta::parse, Token![,])?,
        })
    }
}

impl MacroArgs {
    fn into_config(self) -> MacroConfig {
        let mut config = MacroConfig::default();
        config.shutdown_timeout = "30s".to_string();
        config.double_ctrl_c_window = "2s".to_string();

        for meta in self.args {
            match meta {
                Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("logging") => {
                    match value {
                        Expr::Lit(ExprLit {
                            lit: Lit::Bool(b), ..
                        }) => {
                            config.logging = LoggingConfig::Enabled(b.value);
                        }
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }) => {
                            config.logging = LoggingConfig::Level(s.value());
                        }
                        _ => {
                            // Invalid type - will be caught during validation
                        }
                    }
                }
                Meta::NameValue(MetaNameValue { path, value, .. })
                    if path.is_ident("shutdown_mode") =>
                {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = value
                    {
                        match s.value().as_str() {
                            "single_ctrl_c" => config.shutdown_mode = ShutdownMode::SingleCtrlC,
                            "double_ctrl_c" => config.shutdown_mode = ShutdownMode::DoubleCtrlC,
                            _ => {
                                // Invalid value - will be caught during validation
                            }
                        }
                    }
                }
                Meta::NameValue(MetaNameValue { path, value, .. })
                    if path.is_ident("shutdown_timeout") =>
                {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = value
                    {
                        config.shutdown_timeout = s.value();
                    }
                }
                Meta::NameValue(MetaNameValue { path, value, .. })
                    if path.is_ident("double_ctrl_c_window") =>
                {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = value
                    {
                        config.double_ctrl_c_window = s.value();
                    }
                }
                Meta::NameValue(MetaNameValue { path, value, .. })
                    if path.is_ident("status_fn") =>
                {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = value
                    {
                        if let Ok(path) = syn::parse_str::<syn::Path>(&s.value()) {
                            config.status_fn = Some(path);
                        }
                    }
                }
                _ => {
                    // Unknown attribute - will be caught during validation
                }
            }
        }

        // Validate configuration
        if let Err(e) = validate_config(&config) {
            panic!("Configuration error: {}", e);
        }

        config
    }
}

fn validate_config(config: &MacroConfig) -> Result<(), String> {
    // Validate that status_fn is provided for double_ctrl_c mode
    if matches!(config.shutdown_mode, ShutdownMode::DoubleCtrlC) && config.status_fn.is_none() {
        return Err("status_fn is required when shutdown_mode is 'double_ctrl_c'".to_string());
    }

    // Validate timeout format (basic check)
    if !config.shutdown_timeout.ends_with('s') && !config.shutdown_timeout.ends_with("ms") {
        return Err(format!(
            "shutdown_timeout '{}' must end with 's' or 'ms'",
            config.shutdown_timeout
        ));
    }

    if !config.double_ctrl_c_window.ends_with('s') && !config.double_ctrl_c_window.ends_with("ms") {
        return Err(format!(
            "double_ctrl_c_window '{}' must end with 's' or 'ms'",
            config.double_ctrl_c_window
        ));
    }

    Ok(())
}

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

                    // Trigger graceful shutdown after user main completes
                    fastn_p2p::shutdown().await?;

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
        LoggingConfig::Level(level) => {
            let level_str = level.as_str();
            quote! {
                // Logging setup with custom level and RUST_LOG override
                let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| #level_str.to_string());
                tracing_subscriber::fmt()
                    .with_env_filter(filter)
                    .init();
            }
        }
    }
}

fn generate_shutdown_setup(config: &MacroConfig) -> proc_macro2::TokenStream {
    match config.shutdown_mode {
        ShutdownMode::SingleCtrlC => {
            let _timeout = &config.shutdown_timeout;
            quote! {
                // Single Ctrl+C mode - immediate graceful shutdown
                tokio::spawn(async {
                    tokio::signal::ctrl_c().await.ok();
                    println!("Received Ctrl+C, shutting down gracefully...");

                    // Trigger graceful shutdown
                    if let Err(e) = fastn_p2p::shutdown().await {
                        eprintln!("Graceful shutdown failed: {e}");
                        std::process::exit(1);
                    }

                    std::process::exit(0);
                });
            }
        }
        ShutdownMode::DoubleCtrlC => {
            let status_fn = config
                .status_fn
                .as_ref()
                .expect("status_fn validated during config parsing");
            let window = &config.double_ctrl_c_window;

            quote! {
                // Double Ctrl+C mode - status on first, shutdown on second
                tokio::spawn(async {
                    // Wait for first Ctrl+C
                    tokio::signal::ctrl_c().await.ok();
                    println!("Received first Ctrl+C, showing status...");

                    // Call user's status function
                    #status_fn().await;

                    println!("Press Ctrl+C again within {} to shutdown, or continue running...", #window);

                    // Parse timeout window (basic implementation)
                    let timeout_duration = if #window.ends_with("ms") {
                        let ms: u64 = #window.trim_end_matches("ms").parse().unwrap_or(2000);
                        tokio::time::Duration::from_millis(ms)
                    } else {
                        let s: u64 = #window.trim_end_matches("s").parse().unwrap_or(2);
                        tokio::time::Duration::from_secs(s)
                    };

                    // Wait for second Ctrl+C within timeout window
                    let second_ctrl_c = tokio::time::timeout(
                        timeout_duration,
                        tokio::signal::ctrl_c()
                    ).await;

                    if second_ctrl_c.is_ok() {
                        println!("Received second Ctrl+C, shutting down gracefully...");

                        // Trigger graceful shutdown
                        if let Err(e) = fastn_p2p::shutdown().await {
                            eprintln!("Graceful shutdown failed: {e}");
                            std::process::exit(1);
                        }

                        std::process::exit(0);
                    } else {
                        println!("No second Ctrl+C received within {}, continuing to run...", #window);
                        // Continue running - spawn another signal handler for future Ctrl+C
                        // TODO: This creates a recursive pattern - might need better design
                    }
                });
            }
        }
    }
}
