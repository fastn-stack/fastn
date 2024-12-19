#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_router;

pub mod reader;
mod route;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Router {}

pub enum Method {
    Get,
    Post,
}

#[allow(dead_code)]
// the router will depend on fastn-section.
pub enum Route {
    NotFound,
    // String contains the path, the data may contain more than that was passed to route, e.g., it
    // can extract some extra path-specific data from FASTN.ftd file
    Document(String, serde_json::Value),
    Wasm(String, serde_json::Value),
    Redirect(String),
    Static(String),
}
