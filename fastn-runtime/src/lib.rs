#![allow(dead_code)]

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
mod dom;
mod dom_helpers;
mod element;
mod event;
pub mod html;
mod memory;
mod operation;
pub mod wasm;
mod wasm_helpers;

pub use control::ControlFlow;
pub use document::Document;
pub use dom::{
    Color, DarkModeProperty, Dom, LengthRole, NodeKey, ResponsiveProperty, TextRole,
};
pub use element::{CommonStyle, Container, Dimension, Element, ElementKind, Image, Text};
pub use event::{DomEventKind, ExternalEvent, MouseState};
pub use memory::{
    Closure, ClosurePointer, EventHandler, HeapValue, Memory, Pointer, PointerKey, PointerKind,
    UIProperty,
};
pub use operation::{Operation, Rectangle};

// #[derive(Debug, Default, Clone)]
// pub struct TextStyle {
//     // border: Borders,
//     pub underline: Callable<bool>,
//     pub italic: Callable<bool>,
//     pub strike: Callable<bool>,
//     pub weight: Callable<Option<TextWeight>>,
//     pub color: Callable<Option<fastn_runtime::Color>>,
// }
//
// impl TextStyle {
//     pub fn taffy(&self) -> taffy::style::Style {
//         todo!()
//     }
// }

// #[derive(Debug, Default, Clone)]
// pub struct Callable<T> {
//     pub wat: String,
//     pub refs: Vec<Ref>,
//     pub muts: Vec<Mut>,
//     _t: std::marker::PhantomData<T>,
// }

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
