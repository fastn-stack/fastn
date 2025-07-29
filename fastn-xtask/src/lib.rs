extern crate self as fastn_xtask;

pub mod helpers;
pub use helpers::{Error, Result};
pub mod build_wasm;
pub mod run_template;
pub mod optimise_wasm;
pub mod publish_app;
pub mod update_ui;
pub mod run_ui;
pub mod update_www;
pub mod run_www;
pub mod update_template;

pub fn main() {
    let result: Result<()> = (|| {
        let default_commands = [
            ("build-wasm", "Builds the WASM target from backend."),
            ("run-ui", "Builds and serves the UI for the app, which is served on port 8002."),
            ("update-ui", "Updates UI dependencies for the app, run this only when modifying dependencies in *.fifthtry.site/FASTN.ftd or during the initial setup."),
            ("run-template", "Runs the backend and tests end-to-end functionality of the app."),
            ("update-template", "Updates dependencies for the app's backend template. Run this only when modifying dependencies or during the initial setup."),
            ("run-www", "Serves and tests the public website for the app."),
            ("update-www", "Updates dependencies for the app's public website. Run this only when modifying dependencies or during the initial setup."),
            ("optimise-wasm", "Optimises the generated WASM binary."),
            ("publish-app", "Publishes the app."),
            ("help", "Prints this help message."),
        ];
        let task = std::env::args().nth(1);
        match task.as_deref() {
            Some("build-wasm") => build_wasm::build_wasm()?,
            Some("run-template") => run_template::run_template()?,
            Some("optimise-wasm") => optimise_wasm::optimise_wasm()?,
            Some("publish-app") => publish_app::publish_app()?,
            Some("update-ui") => update_ui::update_ui()?,
            Some("run-ui") => run_ui::run_ui()?,
            Some("update-www") => update_www::update_www()?,
            Some("run-www") => run_www::run_www()?,
            Some("update-template") => update_template::update_template()?,
            _ => print_help(Some(&default_commands)),
        }
        Ok(())
    })();

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

pub fn print_help(commands: Option<&[(&str, &str)]>) {
    eprintln!("fastn xtask CLI");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    cargo xtask <COMMAND>");
    eprintln!();
    eprintln!("COMMANDS:");
    if let Some(cmds) = commands {
        for (command, description) in cmds {
            eprintln!("    {}: {}", command, description);
            eprintln!();
        }
    }
}
