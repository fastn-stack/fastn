#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_router;

mod reader;
mod route;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Default)]
pub struct Router {
    /// name of the current package
    name: String,
    /// list of files in the current package.
    /// note that this is the canonical url: /-/<current-package>/<file>
    /// tho we allow /<file> also with header `Link: </-/<current-package>/<file>>; rel="canonical"`.
    /// for the current package and all dependencies, we store the list of files
    file_list: std::collections::HashMap<String, Vec<String>>,
    redirects: Vec<Redirect>,
    /// only for current package
    dynamic_urls: Vec<DynamicUrl>,
    /// only for current package
    wasm_mounts: Vec<WasmMount>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Redirect {
    source: String,
    destination: String,
    /// source and end can end with *, in which case wildcard will be true
    wildcard: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub enum Fragment {
    Exact(String),
    Argument { kind: Kind, name: String },
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub enum Kind {
    Integer,
    String,
    Boolean,
    Decimal,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct DynamicUrl {
    fragments: Vec<Fragment>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct WasmMount {
    /// any url starting with this
    url: String,
    /// will be handled by this wasm file
    wasm_file: String,
    /// we will remove the url part, and send request to whatever comes after url, but prepended
    /// with wasm_base
    wasm_base: String, // default value /
}

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
