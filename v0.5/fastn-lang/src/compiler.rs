/// this is our main compiler module
///
/// it should be called with a parsed document, and it returns generated JS.
///
/// on success, we return the JS, and list of warnings, and on error, we return the list of
/// diagnostics, which is an enum containing warning and error.
///
/// earlier we had strict mode here, but to simplify things, now we let the caller convert non-empty
/// warnings from OK part as error, and discard the generated JS.
pub async fn compile(
    _symbols: &mut Box<dyn fastn_lang::SymbolStore>,
    document_id: &str,
    source: &str,
) -> Result<fastn_lang::compiler::Output, Error> {
    // this guy will maintain symbols that failed to resolve
    let _d = fastn_unresolved::parse(document_id, source);
    todo!()
}

pub struct Output {
    js: String,
    warnings: Vec<fastn_section::Warning>,
    resolved: Vec<fastn_unresolved::Definition>,
}

// consider:
//
// -- component foo:
// x x:
//
// ... definition skipped ...
//
// -- end: foo
//
// we will store x here. but what if x is actually a type alias to y, and it is y that is changing.
// we have to make sure that we revalidate x when y changes. if we cant do this our entire
// dependency tracking system is useless.
pub struct Error {
    messages: Vec<fastn_section::Diagnostic>,
    // while we failed build the document, we may have successfully resolved some components, and
    // there is no point throwing that work away, and we can use them for the next document.
    //
    // we are not returning vec string (dependencies here), because `Definition::dependencies()` is
    // going to do that.
    resolved: Vec<fastn_unresolved::Definition>,
    failed_symbols: Vec<fastn_unresolved::Identifier, Vec<String>>,
}
