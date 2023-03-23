#[cfg(test)]
#[macro_use]
mod test;

mod code;
mod dummy;
mod element;
mod main;
mod markup;
mod styles;
mod tdoc;
pub(crate) mod utils;
mod value;
mod youtube_id;

pub type FieldWithValue = (ftd::interpreter2::Field, Option<ftd::ast::VariableValue>);

pub use dummy::{DummyElement, ElementConstructor};
pub use element::{
    CheckBox, Code, Column, Common, Container, Document, Element, Event, HTMLData, Iframe, Image,
    ImageSrc, IterativeElement, RawElement, Row, Text, TextInput, WebComponent,
};
pub use main::{ExecuteDoc, RT};
pub use styles::{
    AlignSelf, Alignment, Anchor, Background, BackgroundRepeat, BorderStyle, Color, ColorValue,
    Cursor, FontSize, Length, LineClamp, Loading, Overflow, Region, Resize, Resizing,
    ResponsiveType, Spacing, TextAlign, TextInputType, TextStyle, TextTransform, TextWeight,
    WhiteSpace,
};
pub(crate) use tdoc::TDoc;
pub(crate) use value::Value;

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

    #[error("syntect error: {source}")]
    Syntect {
        #[from]
        source: syntect::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
