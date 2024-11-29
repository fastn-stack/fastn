/// how to resolve local symbols, e.g., inside a function / component
///
/// locals is the stack of locals (excluding globals).
///
/// e.g., inside a function we can have block containing blocks, and each block may have defined
/// some variables, each such nested block is passed as locals,
/// with the innermost block as the last entry.
pub fn resolve(
    _module: &fastn_unresolved::Module,
    // parent: Option<fastn_unresolved::Symbol>,
    name: &mut fastn_unresolved::URIS,
    _definitions: &std::collections::HashMap<fastn_unresolved::Symbol, fastn_unresolved::URD>,
    interner: &mut string_interner::DefaultStringInterner,
    _output: &mut fastn_unresolved::resolver::Output,
    _locals: &[Vec<fastn_unresolved::UR<fastn_unresolved::Argument, fastn_resolved::Argument>>],
) {
    let inner_name = if let fastn_unresolved::UR::UnResolved(name) = name {
        name
    } else {
        return;
    };

    let symbol = match inner_name {
        fastn_section::IdentifierReference::Absolute {
            package,
            module,
            name,
        } => fastn_unresolved::Symbol::new(package.str(), module.str(), name.str(), interner),
        fastn_section::IdentifierReference::Imported { module, name } => {
            if module.str() == "ftd" {
                fastn_unresolved::Symbol::new("ftd", "", name.str(), interner)
            } else {
                // checkout aliases in this module
                todo!()
            }
        }
        _ => todo!(),
    };

    *name = fastn_unresolved::UR::Resolved(symbol);
}
