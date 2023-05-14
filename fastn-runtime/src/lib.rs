extern crate self as fastn_runtime;

/// fastn-surface is a way to describe UI in platform independent way
///
/// fastn-surface::UI is a way to describe arbitrary UI that can be displayed on various backends
/// like in browser, terminal or native. fastn-surface::UI exposes mutation methods, which can be
/// used to mutate the UI once the UI has been rendered on some surface. The mutations are applied
/// in an efficient way.
///
/// fastn-surface::UI also send UI events, like window resize, keyboard, mouse events etc. The
/// event includes data about the event.
#[cfg(feature = "native")]
pub mod wgpu;

mod control;
mod document;
mod element;
mod event;
mod operation;

pub use control::ControlFlow;
pub use document::Document;
pub use element::{Container, Dimension, Element, Image, Text};
pub use event::Event;
pub use operation::{Operation, Rectangle};

slotmap::new_key_type! { pub struct NodeKey; }

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct ColorValue {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: f32,
}

#[derive(Debug, Default, Clone)]
pub struct Color {
    pub light: ColorValue,
    pub dark: ColorValue,
}

#[derive(Debug, Default, Clone)]
pub struct TextStyle {
    pub underline: bool,
    pub italic: bool,
    pub strike: bool,
    pub weight: Option<TextWeight>,
}

#[derive(Debug, Clone)]
pub enum TextWeight {
    EXTRABOLD,
    BOLD,
    SEMIBOLD,
    HEAVY,
    MEDIUM,
    REGULAR,
    LIGHT,
    EXTRALIGHT,
    HAIRLINE,
}
