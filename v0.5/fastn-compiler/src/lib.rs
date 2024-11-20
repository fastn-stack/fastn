#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_compiler;

mod builtins;
mod compiler;
mod js;
mod symbols;

pub use builtins::BUILTINS;
pub use compiler::compile;
pub(crate) use compiler::Compiler;
pub use fastn_section::Result;
pub use symbols::SymbolStore;

pub struct Output {
    #[expect(unused)]
    js: String,
    #[expect(unused)]
    warnings: Vec<fastn_section::Spanned<fastn_section::Warning>>,
    #[expect(unused)]
    resolved: Vec<fastn_resolved::Definition>,
    // should we also return / cache partially resolved symbols?
}

pub struct Error {
    #[expect(unused)]
    messages: Vec<fastn_section::Diagnostic>,
    /// while we failed build the document, we may have successfully resolved some components, and
    /// there is no point throwing that work away, and we can use them for the next document.
    ///
    /// we are not returning vec string (dependencies here), because `Definition::dependencies()` is
    /// going to do that.
    #[expect(unused)]
    resolved: Vec<fastn_resolved::Definition>,
    /// while parsing, we found some symbols are wrong, e.g., say the document tried to use component `
    /// foo` but `foo` internally is calling `bar`, and there is no such component, and say `foo` is
    /// trying to use `baz` as a type, but there is no such type. Anyone else trying to use `foo`
    /// will also fail, so we store these errors here.
    ///
    /// also, consider:
    ///
    /// -- component foo:
    /// x x:
    ///
    /// ... definition skipped ...
    ///
    /// -- end: foo
    ///
    /// we will store x here. but what if x is actually a type alias to y, and it is y that is
    /// changing. we have to make sure that we revalidate x when y changes. if we cant do this, our
    /// entire dependency tracking system is useless.
    #[expect(unused)]
    symbol_errors: Vec<SymbolError>,
}

/// a symbol can fail because of multiple errors, and we will store the various ones in the
pub struct SymbolError {
    #[expect(unused)]
    symbol: fastn_unresolved::Identifier,
    #[expect(unused)]
    dependencies: Vec<String>,
    /// this is all the errors that came when trying to resolve this symbol.
    #[expect(unused)]
    errors: Vec<fastn_section::Error>,
}
