extern crate self as fastn_xtask;

pub mod helpers;

pub mod build_wasm;
pub mod run_template;
pub mod optimise_wasm;
pub mod publish_app;
pub mod update_ui;
pub mod run_ui;
pub mod update_www;
pub mod run_www;
pub mod update_template;


pub fn main() -> Result<(), String> {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        Some("build-wasm") => build_wasm::build_wasm().map_err(|e| e.to_string())?,
        Some("run-template") => run_template::run_template().map_err(|e| e.to_string())?,
        Some("optimise-wasm") => optimise_wasm::optimise_wasm().map_err(|e| e.to_string())?,
        Some("publish-app") => publish_app::publish_app().map_err(|e| e.to_string())?,
        Some("update-ui") => update_ui::update_ui().map_err(|e| e.to_string())?,
        Some("run-ui") => run_ui::run_ui().map_err(|e| e.to_string())?,
        Some("update-www") => update_www::update_www().map_err(|e| e.to_string())?,
        Some("run-www") => run_www::run_www().map_err(|e| e.to_string())?,
        Some("update-template") => update_template::update_template().map_err(|e| e.to_string())?,
        _ => print_help(None),
    }

    Ok(())
}

pub fn print_help(additional_commands: Option<&[(&str, &str)]>) {
    eprintln!("fastn xtask CLI");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    cargo xtask <COMMAND>");
    eprintln!();
    eprintln!("COMMANDS:");
    
    eprintln!("    build-wasm: Builds the WASM target from backend.");
    eprintln!();
    eprintln!("    run-ui: Builds and serves the UI for the lets-XXX app, which is served on port 8002.");
    eprintln!();
    eprintln!("    update-ui: Updates UI dependencies for the lets-XXX app, run this only when modifying dependencies in lets-XXX.fifthtry.site/FASTN.ftd or during the initial setup.");
    eprintln!();
    eprintln!("    run-template: Runs the backend and tests end-to-end functionality of the lets-XXX app.");
    eprintln!();
    eprintln!("    update-template: Updates dependencies for the lets-XXX app's backend template. Run this only when modifying dependencies or during the initial setup.");
    eprintln!();
    eprintln!("    run-www: Serves and tests the public website for the lets-XXX app.");
    eprintln!();
    eprintln!("    update-www: Updates dependencies for the lets-XXX app's public website. Run this only when modifying dependencies or during the initial setup.");
    eprintln!();
    eprintln!("    optimise-wasm: Optimises the generated WASM binary.");
    eprintln!();
    eprintln!("    publish-app: Publishes the lets-XXX app.");
    eprintln!();
    
    if let Some(additional) = additional_commands {
        for (command, description) in additional {
            eprintln!("    {}: {}", command, description);
            eprintln!();
        }
    }
    
    eprintln!("    help: Prints this help message.");
}
