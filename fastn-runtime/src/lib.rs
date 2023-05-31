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
mod element;
mod event;
mod memory;
mod operation;
pub mod wasm;
mod wasm_helpers;

pub use control::ControlFlow;
pub use document::Document;
pub use dom::Dom;
pub use element::{CommonStyleMinusTaffy, Container, Dimension, Element, Image, Text};
pub use event::Event;
pub use memory::{Closure, Memory, Pointer, PointerKind, UIProperty};
pub use operation::{Operation, Rectangle};

slotmap::new_key_type! { pub struct NodeKey; }
slotmap::new_key_type! { pub struct PointerKey; }
slotmap::new_key_type! { pub struct ClosurePointer; }

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
    // border: Borders,
    pub underline: Callable<bool>,
    pub italic: Callable<bool>,
    pub strike: Callable<bool>,
    pub weight: Callable<Option<TextWeight>>,
    pub color: Callable<Option<fastn_runtime::Color>>,
}

impl TextStyle {
    pub fn taffy(&self) -> taffy::style::Style {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Ref;

#[derive(Debug, Default, Clone)]
pub struct Mut;

#[derive(Debug, Default, Clone)]
pub struct Callable<T> {
    pub wat: String,
    pub refs: Vec<Ref>,
    pub muts: Vec<Mut>,
    _t: std::marker::PhantomData<T>,
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
