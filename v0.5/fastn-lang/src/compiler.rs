/// this is our main compiler module
///
/// it should be called with a parsed document, and it returns generated JS.
///
/// on success, we return the JS, and list of warnings, and on error, we return the list of
/// diagnostics, which is an enum containing warning and error.
///
/// earlier we had strict mode here, but to simplify things, now we let the caller convert non-empty
/// warnings from OK part as error, and discard the generated JS.
pub async fn compile<'input>(
    symbols: &mut Box<dyn fastn_lang::SymbolStore<'input>>,
    document_id: &str,
    source: &str,
    _auto_imports: &[fastn_section::AutoImport],
) -> Result<fastn_lang::compiler::Output, fastn_lang::compiler::Error> {
    // this guy will maintain symbols that failed to resolve, along with their dependencies, or maybe
    // just the one dependency that failed?
    let mut d = fastn_unresolved::parse(document_id, source);
    let mut bag = std::collections::HashMap::new();

    // we only make 10 attempts to resolve the document
    for _ in 1..10 {
        // resolve_document can internally run in parallel.
        let (mut unresolved_symbols, partially_resolved) = resolve_document(&mut d, &bag);
        update_partially_resolved(&mut bag, partially_resolved);
        if unresolved_symbols.is_empty() {
            break;
        }

        fetch_unresolved_symbols(symbols, &mut d, &mut bag, &mut unresolved_symbols).await;
        // this itself has to happen in a loop
        for _ in 1..10 {
            // resolve_document can internally run in parallel.
            let partially_resolved = resolve_symbols(&mut d, &bag, &mut unresolved_symbols);
            update_partially_resolved(&mut bag, partially_resolved);

            if unresolved_symbols.is_empty() {
                break;
            }
            fetch_unresolved_symbols(symbols, &mut d, &mut bag, &mut unresolved_symbols).await;
        }

        if !unresolved_symbols.is_empty() {
            // we were not able to resolve all symbols
        }
    }

    todo!()
}

fn update_partially_resolved(
    _bag: &mut std::collections::HashMap<String, fastn_lang::LookupResult<'_>>,
    _partially_resolved: Vec<fastn_unresolved::Definition>,
) {
    todo!()
}

async fn fetch_unresolved_symbols<'input>(
    _symbols: &mut Box<dyn fastn_lang::SymbolStore<'input>>,
    _d: &mut fastn_unresolved::Document,
    _bag: &mut std::collections::HashMap<String, fastn_lang::LookupResult<'input>>,
    _symbols_to_fetch: &mut [fastn_unresolved::SymbolName],
) {
    todo!()
}

/// try to resolve as many symbols as possible, and return the ones that we made any progress on.
///
/// this function should be called in a loop, until list of symbols is empty.
fn resolve_symbols(
    _d: &mut fastn_unresolved::Document,
    _bag: &std::collections::HashMap<String, fastn_lang::LookupResult>,
    _symbols: &mut [fastn_unresolved::SymbolName],
) -> Vec<fastn_unresolved::Definition> {
    todo!()
}

/// try to make as much progress as possibly by resolving as many symbols as possible, and return
/// the vec of ones that could not be resolved.
///
/// it also returns vec of partially resolved symbols, so we do not directly modify the bag, we want
/// all bag updates to happen in one place.
///
/// if this returns an empty list of symbols, we can go ahead and generate the JS.
fn resolve_document(
    d: &mut fastn_unresolved::Document,
    _bag: &std::collections::HashMap<String, fastn_lang::LookupResult>,
) -> (
    Vec<fastn_unresolved::SymbolName>,
    Vec<fastn_unresolved::Definition>,
) {
    for ci in &d.content {
        if let fastn_unresolved::UR::UnResolved(_c) = ci {
            todo!()
        }
    }

    todo!()
}

pub struct Output {
    #[expect(unused)]
    js: String,
    #[expect(unused)]
    warnings: Vec<fastn_section::Warning>,
    #[expect(unused)]
    resolved: Vec<fastn_type::Definition>,
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
    resolved: Vec<fastn_type::Definition>,
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
