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
    symbols: &mut Box<dyn fastn_compiler::SymbolStore>,
    document_id: &fastn_unresolved::ModuleName,
    source: &str,
    _auto_imports: &[fastn_section::AutoImport],
) -> Result<fastn_compiler::Output, fastn_compiler::Error> {
    let mut interner = string_interner::StringInterner::new();
    // this guy will maintain symbols that failed to resolve, along with their dependencies, or maybe
    // just the one dependency that failed?
    let mut d = fastn_unresolved::parse(document_id, source);
    let mut bag = std::collections::HashMap::new();

    // we only make 10 attempts to resolve the document: we need a warning if we are not able to
    // resolve the document in 10 attempts.
    for _ in 1..10 {
        // resolve_document can internally run in parallel.
        let (mut unresolved_symbols, partially_resolved) = resolve_document(&mut d, &bag);
        update_partially_resolved(&mut bag, partially_resolved);
        if unresolved_symbols.is_empty() {
            break;
        }

        fetch_unresolved_symbols(symbols, &mut bag, &mut unresolved_symbols, &mut interner).await;
        // this itself has to happen in a loop. we need a warning if we are not able to resolve all
        // symbols in 10 attempts.
        for _ in 1..10 {
            // resolve_document can internally run in parallel.
            let partially_resolved = resolve_symbols(&mut d, &bag, &mut unresolved_symbols);
            update_partially_resolved(&mut bag, partially_resolved);

            if unresolved_symbols.is_empty() {
                break;
            }
            fetch_unresolved_symbols(symbols, &mut bag, &mut unresolved_symbols, &mut interner)
                .await;
        }

        if !unresolved_symbols.is_empty() {
            // we were not able to resolve all symbols
        }
    }

    todo!()
}

fn update_partially_resolved(
    _bag: &mut std::collections::HashMap<
        fastn_unresolved::SymbolName,
        fastn_compiler::LookupResult,
    >,
    _partially_resolved: Vec<fastn_unresolved::Definition>,
) {
    todo!()
}

async fn fetch_unresolved_symbols(
    symbols: &mut Box<dyn fastn_compiler::SymbolStore>,
    _bag: &mut std::collections::HashMap<
        fastn_unresolved::SymbolName,
        fastn_compiler::LookupResult,
    >,
    symbols_to_fetch: &mut [fastn_unresolved::SymbolName],
    interner: &mut string_interner::DefaultStringInterner,
) {
    let _found = symbols.lookup(interner, symbols_to_fetch);
}

/// try to resolve as many symbols as possible, and return the ones that we made any progress on.
///
/// this function should be called in a loop, until list of symbols is empty.
fn resolve_symbols(
    _d: &mut fastn_unresolved::Document,
    _bag: &std::collections::HashMap<fastn_unresolved::SymbolName, fastn_compiler::LookupResult>,
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
    _bag: &std::collections::HashMap<fastn_unresolved::SymbolName, fastn_compiler::LookupResult>,
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
