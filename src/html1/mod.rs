#[cfg(test)]
#[macro_use]
mod test;

mod events;
mod main;
pub mod utils;

pub use events::Action;
pub use main::HtmlUI;
