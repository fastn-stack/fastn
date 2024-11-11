#[cfg(test)]
#[macro_use]
mod test;

mod data;
mod dependencies;
mod dummy_html;
mod events;
mod fastn_type_functions;
mod functions;
mod main;
pub mod utils;
mod variable_dependencies;

pub(crate) use dummy_html::{DummyHtmlGenerator, HelperHtmlGenerator};
pub use events::Action;
pub use functions::{ExpressionGenerator, FunctionGenerator};
pub(crate) use main::{escape, RawHtmlGenerator};
pub use main::{HTMLData, HtmlUI};
pub use variable_dependencies::VariableDependencyGenerator;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("InterpreterError: {}", _0)]
    InterpreterError(#[from] ftd::interpreter::Error),

    #[error("{doc_id}:{line_number} -> {message}")]
    ParseError {
        message: String,
        doc_id: String,
        line_number: usize,
    },

    #[error("InterpretEvalexprErrorerError: {}", _0)]
    EvalexprError(#[from] fastn_grammar::evalexpr::EvalexprError),
}

pub type Result<T> = std::result::Result<T, Error>;
