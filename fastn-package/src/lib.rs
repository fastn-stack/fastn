extern crate self as fastn_package;

mod initialize;
pub mod initializer;
pub mod old_fastn;
pub(crate) mod sqlite;

pub use initialize::initialize;

const FASTN_PACKAGE_VARIABLE: &str = "fastn#package";

static FTD_CACHE: tokio::sync::OnceCell<
    tokio::sync::RwLock<std::collections::HashMap<String, ftd_ast::AST>>,
> = tokio::sync::OnceCell::const_new();

pub fn fastn_ftd_2023() -> &'static str {
    include_str!("../fastn_2023.ftd")
}
