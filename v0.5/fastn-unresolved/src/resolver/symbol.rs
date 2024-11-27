/// how to resolve local symbols, e.g., inside a function / component
///
/// locals is the stack of locals (excluding globals).
///
/// e.g., inside a function we can have block containing blocks, and each block may have defined
/// some variables, each such nested block is passed as locals,
/// with the innermost block as the last entry.
pub fn resolve(
    _module: &fastn_unresolved::Module,
    name: &mut fastn_unresolved::URIS,
    _input: &fastn_unresolved::resolver::Input<'_>,
    output: &mut fastn_unresolved::resolver::Output,
    _locals: &[Vec<fastn_unresolved::UR<fastn_unresolved::Argument, fastn_resolved::Argument>>],
) {
    let inner_name = if let fastn_unresolved::UR::UnResolved(name) = name {
        name
    } else {
        return;
    };

    let _n = match parse(inner_name.str()) {
        Ok(n) => n,
        Err(e) => {
            output.errors.push(inner_name.spanned(e));
            return;
        }
    };

    *name = fastn_unresolved::UR::Resolved(todo!());
}

fn parse(s: &str) -> Result<N, fastn_section::Error> {
    // inner_name can be either:
    //   - foo: component defined in current module or import-exposed,
    //   - bar.foo: component defined in module bar,
    //   - bar#foo: component using the absolute path.
    //
    // our job is to verify that in case, such a symbol is defined, and return the full symbol name
    Ok(N::Local(s))
}

enum N<'a> {
    // foo
    Local(&'a str),
    // bar.foo: module = bar, name: foo
    Imported { module: &'a str, name: &'a str },
    Absolute(&'a str),
}
