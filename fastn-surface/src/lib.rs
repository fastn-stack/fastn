extern crate self as fastn_surface;

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

mod element;
mod document;

pub use element::{Container, Dimension, Element, Image, Text};
pub use document::Document;

slotmap::new_key_type! { pub struct NodeKey; }

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct ColorValue {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub alpha: f32,
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Color {
    pub light: ColorValue,
    pub dark: ColorValue,
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct TextStyle {
    pub underline: bool,
    pub italic: bool,
    pub strike: bool,
    pub weight: Option<TextWeight>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
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
