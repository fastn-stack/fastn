extern crate self as fastn_package;

mod initialize;
pub mod initializer;
pub mod old_fastn;
pub(crate) mod sqlite;

pub use initialize::initialize;

const FASTN_PACKAGE_VARIABLE: &str = "fastn#package";

static FTD_CACHE: once_cell::sync::Lazy<
    tokio::sync::RwLock<std::collections::HashMap<String, ftd::ast::AST>>,
> = once_cell::sync::Lazy::new(|| tokio::sync::RwLock::new(std::collections::HashMap::new()));
