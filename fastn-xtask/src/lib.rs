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