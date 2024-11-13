#[cfg(test)]
#[macro_use]
mod test;

pub mod code;
mod dummy;
mod element;
mod fastn_type_functions;
mod main;
mod markup;
mod rive;
mod styles;
mod tdoc;
pub(crate) mod utils;
pub mod value;
mod youtube_id;

pub type FieldWithValue = (fastn_type::Field, Option<ftd_ast::VariableValue>);

pub use dummy::{DummyElement, ElementConstructor};
pub use element::{
    CheckBox, Code, Column, Common, Container, ContainerElement, Document, Element, Event,
    HTMLData, Iframe, Image, ImageSrc, IterativeElement, RawElement, RawImage, Rive, Row, Text,
    TextInput, WebComponent,
};
pub use main::{Device, ExecuteDoc, RT};
pub use rive::RiveData;
pub use styles::{
    AlignSelf, Alignment, Anchor, Background, BackgroundImage, BackgroundPosition,
    BackgroundRepeat, BackgroundSize, BorderStyle, BreakpointWidth, Color, ColorValue, Cursor,
    Display, FontSize, ImageFit, Length, LineClamp, LinearGradient, LinearGradientColor,
    LinearGradientDirection, Loading, Overflow, Region, Resize, Resizing, ResponsiveType, Shadow,
    Spacing, TextAlign, TextInputType, TextStyle, TextTransform, TextWeight, WhiteSpace,
};
pub(crate) use tdoc::TDoc;
pub(crate) use value::Value;

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

    #[error("syntect error: {source}")]
    Syntect {
        #[from]
        source: syntect::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
