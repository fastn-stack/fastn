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
    _output: &mut fastn_unresolved::resolver::Output,
    _locals: &[Vec<fastn_unresolved::UR<fastn_unresolved::Argument, fastn_resolved::Argument>>],
) {
    let _inner_name = if let fastn_unresolved::UR::UnResolved(name) = name {
        name
    } else {
        return;
    };

    *name = fastn_unresolved::UR::Resolved(todo!());
}
