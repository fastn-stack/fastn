#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_router;

pub mod reader;
mod route;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Router {
    file_list: Vec<String>,
    redirects: Vec<(String, String)>,
    dynamic_urls: Vec<DynamicUrl>,
    wasm_mounts: Vec<WasmMount>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct DynamicUrl {}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct WasmMount {}

#[derive(Debug, Copy, PartialEq, Clone)]
pub enum Method {
    Get,
    Post,
}

#[allow(dead_code)]
#[derive(Debug)]
// the router will depend on fastn-section.
pub enum Route {
    /// not found tells you which ftd document to serve as not found page
    NotFound(Document),
    // String contains the path, the data may contain more than that was passed to route, e.g., it
    // can extract some extra path-specific data from FASTN.ftd file
    Document(Document),
    Wasm {
        wasm_file: String,
        not_found: Document,
    },
    Redirect(String),
    /// we return the not found document as well in case the static file is missing
    Static {
        package: String,
        path: String,
        mime: String,
        not_found: Document,
    },
}

#[derive(Debug)]
pub struct Document {
    // this is private yet
    #[expect(unused)]
    pub(crate) path: String,
    #[expect(unused)]
    pub(crate) partial: serde_json::Value,
    #[expect(unused)]
    pub(crate) keys: Vec<String>,
}

#[derive(Debug)]
pub enum RouterError {}

impl Document {
    pub fn with_data(
        self,
        _data: &[u8],
    ) -> Result<(String, serde_json::Map<String, serde_json::Value>), RouterError> {
        todo!()
    }
}
