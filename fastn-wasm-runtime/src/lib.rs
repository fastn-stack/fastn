#![allow(dead_code)]
#![deny(unused_crate_dependencies)]

extern crate self as fastn_runtime;

/// fastn-wasm-runtime is a way to describe UI in platform independent way
///
/// fastn-wasm-runtime::Dom is a way to describe arbitrary UI that can be displayed on various backends
/// like in browser, terminal or native. fastn-surface::Dom exposes mutation methods, which can be
/// used to mutate the UI once the UI has been rendered on some surface. The mutations are applied
/// in an efficient way.
///
/// fastn-surface::UI also send UI events, like window resize, keyboard, mouse events etc. The
/// event includes data about the event.

#[cfg(feature = "native")]
pub mod wgpu;

mod control;
#[cfg(not(feature = "browser"))]
mod document;
mod dom;
mod element;
mod event;
mod memory;
#[cfg(not(feature = "browser"))]
mod operation;
#[cfg(any(feature = "native", feature = "terminal"))]
mod renderable;
#[cfg(feature = "server")]
mod server;
#[cfg(not(feature = "browser"))]
pub mod wasm;
#[cfg(not(feature = "browser"))]
mod wasm_helpers;
#[cfg(feature = "browser")]
mod web;

pub use control::ControlFlow;
#[cfg(not(feature = "browser"))]
pub use document::Document;
#[cfg(not(feature = "browser"))]
pub use dom::Dom;
pub use dom::{node_key_to_id, DomT, NodeKey};
pub use element::{CommonStyle, Container, Dimension, Element, ElementKind, Image, Text};
pub use event::{DomEventKind, ExternalEvent, MouseState};
pub use memory::heap::{Attachment, Heap, HeapData, HeapValue};
pub use memory::pointer::{ClosurePointer, Pointer, PointerKey, PointerKind};
pub use memory::ui::{
    Color, DarkModeProperty, DynamicProperty, LengthRole, ResponsiveProperty, TextRole, UIProperty,
};
pub use memory::{Closure, EventHandler, Frame, Memory};
#[cfg(not(feature = "browser"))]
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
