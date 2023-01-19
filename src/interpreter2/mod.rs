#[cfg(test)]
#[macro_use]
mod test;

#[macro_export]
macro_rules! try_ok_state {
    ($e:expr) => {
        match $e {
            $crate::interpreter2::StateWithThing::State(s) => {
                return Ok($crate::interpreter2::StateWithThing::new_state(s))
            }
            $crate::interpreter2::StateWithThing::Continue => {
                return Ok($crate::interpreter2::StateWithThing::new_continue())
            }
            $crate::interpreter2::StateWithThing::Thing(t) => t,
        }
    };
}

#[macro_export]
macro_rules! try_state {
    ($e:expr) => {
        match $e {
            $crate::interpreter2::StateWithThing::State(s) => {
                return $crate::interpreter2::StateWithThing::new_state(s)
            }
            $crate::interpreter2::StateWithThing::Continue => {
                return $crate::interpreter2::StateWithThing::new_continue()
            }
            $crate::interpreter2::StateWithThing::Thing(t) => t,
        }
    };
}

mod constants;
mod main;
mod main2;
mod tdoc;
mod things;
pub mod utils;

pub use constants::{
    FTD_ALIGN, FTD_ALIGN_BOTTOM_CENTER, FTD_ALIGN_BOTTOM_LEFT, FTD_ALIGN_BOTTOM_RIGHT,
    FTD_ALIGN_CENTER, FTD_ALIGN_LEFT, FTD_ALIGN_RIGHT, FTD_ALIGN_SELF, FTD_ALIGN_SELF_CENTER,
    FTD_ALIGN_SELF_END, FTD_ALIGN_SELF_START, FTD_ALIGN_TOP_CENTER, FTD_ALIGN_TOP_LEFT,
    FTD_ALIGN_TOP_RIGHT, FTD_ANCHOR, FTD_ANCHOR_PARENT, FTD_ANCHOR_WINDOW, FTD_BACKGROUND,
    FTD_BACKGROUND_COLOR, FTD_BACKGROUND_SOLID, FTD_BREAKPOINT_WIDTH, FTD_BREAKPOINT_WIDTH_DATA,
    FTD_BTB, FTD_COLOR, FTD_COLOR_LIGHT, FTD_COLOR_SCHEME, FTD_CTA_COLOR, FTD_CURSOR,
    FTD_CURSOR_ALIAS, FTD_CURSOR_ALL_SCROLL, FTD_CURSOR_CELL, FTD_CURSOR_COL_RESIZE,
    FTD_CURSOR_CONTEXT_MENU, FTD_CURSOR_COPY, FTD_CURSOR_CROSSHAIR, FTD_CURSOR_DEFAULT,
    FTD_CURSOR_EW_RESIZE, FTD_CURSOR_E_RESIZE, FTD_CURSOR_GRAB, FTD_CURSOR_GRABBING,
    FTD_CURSOR_HELP, FTD_CURSOR_MOVE, FTD_CURSOR_NESW_RESIZE, FTD_CURSOR_NE_RESIZE,
    FTD_CURSOR_NONE, FTD_CURSOR_NOT_ALLOWED, FTD_CURSOR_NO_DROP, FTD_CURSOR_NS_RESIZE,
    FTD_CURSOR_NWSE_RESIZE, FTD_CURSOR_NW_RESIZE, FTD_CURSOR_N_RESIZE, FTD_CURSOR_POINTER,
    FTD_CURSOR_PROGRESS, FTD_CURSOR_ROW_RESIZE, FTD_CURSOR_SE_RESIZE, FTD_CURSOR_SW_RESIZE,
    FTD_CURSOR_S_RESIZE, FTD_CURSOR_TEXT, FTD_CURSOR_VERTICAL_TEXT, FTD_CURSOR_WAIT,
    FTD_CURSOR_W_RESIZE, FTD_CURSOR_ZOOM_IN, FTD_CURSOR_ZOOM_OUT, FTD_CUSTOM_COLORS, FTD_DEVICE,
    FTD_DEVICE_DATA, FTD_DEVICE_DATA_DESKTOP, FTD_DEVICE_DATA_MOBILE, FTD_FONT_SIZE,
    FTD_FONT_SIZE_EM, FTD_FONT_SIZE_PX, FTD_FONT_SIZE_REM, FTD_IMAGE_SRC, FTD_IMAGE_SRC_DARK,
    FTD_IMAGE_SRC_LIGHT, FTD_LENGTH, FTD_LENGTH_CALC, FTD_LENGTH_EM, FTD_LENGTH_PERCENT,
    FTD_LENGTH_PX, FTD_LENGTH_REM, FTD_LENGTH_RESPONSIVE, FTD_LENGTH_VH, FTD_LENGTH_VW,
    FTD_LOADING, FTD_LOADING_EAGER, FTD_LOADING_LAZY, FTD_OVERFLOW, FTD_OVERFLOW_AUTO,
    FTD_OVERFLOW_HIDDEN, FTD_OVERFLOW_SCROLL, FTD_OVERFLOW_VISIBLE, FTD_PST, FTD_REGION,
    FTD_REGION_H1, FTD_REGION_H2, FTD_REGION_H3, FTD_REGION_H4, FTD_REGION_H5, FTD_REGION_H6,
    FTD_RESIZE, FTD_RESIZE_BOTH, FTD_RESIZE_HORIZONTAL, FTD_RESIZE_VERTICAL, FTD_RESIZING,
    FTD_RESIZING_AUTO, FTD_RESIZING_FILL_CONTAINER, FTD_RESIZING_FIXED, FTD_RESIZING_HUG_CONTENT,
    FTD_RESPONSIVE_LENGTH, FTD_RESPONSIVE_LENGTH_DESKTOP, FTD_RESPONSIVE_TYPE,
    FTD_RESPONSIVE_TYPE_DESKTOP, FTD_SPACING_MODE, FTD_SPACING_MODE_SPACE_AROUND,
    FTD_SPACING_MODE_SPACE_BETWEEN, FTD_SPACING_MODE_SPACE_EVENLY, FTD_SPECIAL_VALUE,
    FTD_TEXT_ALIGN, FTD_TEXT_ALIGN_CENTER, FTD_TEXT_ALIGN_END, FTD_TEXT_ALIGN_JUSTIFY,
    FTD_TEXT_ALIGN_START, FTD_TEXT_INPUT_TYPE, FTD_TEXT_INPUT_TYPE_EMAIL,
    FTD_TEXT_INPUT_TYPE_PASSWORD, FTD_TEXT_INPUT_TYPE_TEXT, FTD_TEXT_INPUT_TYPE_URL,
    FTD_TEXT_TRANSFORM, FTD_TEXT_TRANSFORM_CAPITALIZE, FTD_TEXT_TRANSFORM_INHERIT,
    FTD_TEXT_TRANSFORM_INITIAL, FTD_TEXT_TRANSFORM_LOWERCASE, FTD_TEXT_TRANSFORM_NONE,
    FTD_TEXT_TRANSFORM_UPPERCASE, FTD_TYPE, FTD_TYPE_DATA, FTD_WHITESPACE,
    FTD_WHITESPACE_BREAKSPACES, FTD_WHITESPACE_NORMAL, FTD_WHITESPACE_NOWRAP, FTD_WHITESPACE_PRE,
    FTD_WHITESPACE_PRELINE, FTD_WHITESPACE_PREWRAP,
};
pub use main2::{
    interpret, interpret_with_line_number, Document, Interpreter, InterpreterState,
    InterpreterWithoutState, StateWithThing, ToProcess,
};
pub use tdoc::TDoc;
pub use things::{
    component::{
        Argument, Component, ComponentDefinition, Event, EventName, Loop, Property, PropertySource,
    },
    default,
    expression::Expression,
    function::{Function, FunctionCall},
    kind::{Kind, KindData},
    or_type::{OrType, OrTypeVariant},
    record::{Field, Record},
    value::{PropertyValue, PropertyValueSource, Value},
    variable::{ConditionalValue, Variable},
    Thing,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("P1Error: {}", _0)]
    P1Error(#[from] ftd::p11::Error),

    #[error("OldP1Error: {}", _0)]
    OldP1Error(#[from] ftd::p1::Error),

    #[error("ASTError: {}", _0)]
    ASTError(#[from] ftd::ast::Error),

    #[error("InvalidKind: {doc_id}:{line_number} -> {message}")]
    InvalidKind {
        doc_id: String,
        line_number: usize,
        message: String,
    },

    #[error("ValueNotFound: {doc_id}:{line_number} -> {message}")]
    ValueNotFound {
        doc_id: String,
        line_number: usize,
        message: String,
    },

    #[error("ParseIntError: {}", _0)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("ParseFloatError: {}", _0)]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error("ParseBoolError: {}", _0)]
    ParseBoolError(#[from] std::str::ParseBoolError),

    #[error("{doc_id}:{line_number} -> {message}")]
    ParseError {
        message: String,
        doc_id: String,
        line_number: usize,
    },

    #[error("EvalexprError: {}", _0)]
    EvalexprError(#[from] ftd::evalexpr::EvalexprError),

    #[error("serde error: {source}")]
    Serde {
        #[from]
        source: serde_json::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
