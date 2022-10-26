#[cfg(test)]
#[macro_use]
mod test;

mod data;
mod events;
mod functions;
mod main;
pub mod utils;

pub use events::Action;
pub use functions::FunctionGenerator;
pub use main::HtmlUI;
