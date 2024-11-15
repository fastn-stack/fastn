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
    _symbols: &mut Box<dyn fastn_lang::SymbolStore<'input>>,
    document_id: &str,
    source: &str,
    _auto_imports: &[fastn_section::AutoImport],
) -> Result<fastn_lang::compiler::Output, fastn_lang::compiler::Error> {
    // this guy will maintain symbols that failed to resolve, along with their dependencies, or maybe
    // just the one dependency that failed?
    let mut d = fastn_unresolved::parse(document_id, source);
    let mut bag = std::collections::HashMap::new();

    for _ in 1..10 {
        // we only make 10 attempts to resolve the document
        let mut symbols = resolve_document(&mut d, &mut bag);
        if symbols.is_empty() {
            break;
        }
        // this itself has to happen in a loop
        for _ in 1..10 {
            // TODO: fetch symbols from
            resolve_symbols(&mut d, &mut bag, &mut symbols);
            if symbols.is_empty() {
                break;
            }
        }
    }

    todo!()
}

fn resolve_symbols(
    _d: &mut fastn_unresolved::Document,
    _bag: &mut std::collections::HashMap<String, fastn_lang::LookupResult>,
    _symbols: &mut [fastn_unresolved::SymbolName],
) -> Vec<fastn_unresolved::SymbolName> {
    todo!()
}

fn resolve_document(
    d: &mut fastn_unresolved::Document,
    _bag: &mut std::collections::HashMap<String, fastn_lang::LookupResult>,
) -> Vec<fastn_unresolved::SymbolName> {
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
    resolved: Vec<fastn_unresolved::Definition>,
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
    resolved: Vec<fastn_unresolved::Definition>,
    /// while parsing we found some symbols are wrong, e.g., say the document tried to use component `
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
    /// we will store x here. but what if x is actually a type alias to y, and it is y that is changing.
    /// we have to make sure that we revalidate x when y changes. if we cant do this our entire
    /// dependency tracking system is useless.
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
