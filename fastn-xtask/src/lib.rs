use std::env;

pub mod new_app;
pub use new_app::new_app;

pub mod build_wasm;
pub use build_wasm::build_wasm;

pub mod run_template;
pub use run_template::run_template;

pub mod optimise_wasm;
pub use optimise_wasm::optimise_wasm;

pub mod publish_app;
pub use publish_app::publish_app;

pub mod update_ui;
pub use update_ui::update_ui;

pub mod run_ui;
pub use run_ui::run_ui;

pub mod update_www;
pub use update_www::update_www;

pub mod run_www;
pub use run_www::run_www;

pub mod update_template;
pub use update_template::update_template;


pub fn cli() -> Result<(), String> {
    let task = env::args().nth(1);
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
        _ => print_help(),
    }

    Ok(())
}

fn print_help() {
    eprintln!(
        r#"fastn xtask CLI

USAGE:
    cargo xtask <COMMAND>

COMMANDS:
    build-wasm: Builds the WASM target from backend.

    run-ui: Builds and serves the UI for the lets-XXX app, which is served on port 8002.

    update-ui: Updates UI dependencies for the lets-XXX app, run this only when modifying dependencies in lets-XXX.fifthtry.site/FASTN.ftd or during the initial setup.

    run-template: Runs the backend and tests end-to-end functionality of the lets-XXX app.

    update-template: Updates dependencies for the lets-XXX app's backend template. Run this only when modifying dependencies or during the initial setup.

    run-www: Serves and tests the public website for the lets-XXX app.

    update-www: Updates dependencies for the lets-XXX app's public website. Run this only when modifying dependencies or during the initial setup.

    optimise-wasm: Optimises the generated WASM binary.

    publish-app: Publishes the lets-XXX app.

    help: Prints this help message.
"#
    )
}
