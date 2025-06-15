pub mod template;
pub use template::new_app;

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
