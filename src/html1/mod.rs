#[cfg(test)]
#[macro_use]
mod test;

mod data;
mod dependencies;
mod events;
mod functions;
mod main;
pub mod utils;

pub use events::Action;
pub use functions::FunctionGenerator;
pub use main::HtmlUI;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("InterpreterError: {}", _0)]
    InterpreterError(#[from] ftd::interpreter2::Error),

    #[error("{doc_id}:{line_number} -> {message}")]
    ParseError {
        message: String,
        doc_id: String,
        line_number: usize,
    },

    #[error("InterpretEvalexprErrorerError: {}", _0)]
    EvalexprError(#[from] evalexpr::EvalexprError),
}

pub type Result<T> = std::result::Result<T, Error>;
