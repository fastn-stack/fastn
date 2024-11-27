pub fn resolve(
    _module: &fastn_unresolved::Symbol,
    name: &mut fastn_unresolved::URIS,
    _input: &fastn_unresolved::resolver::Input<'_>,
    _output: &mut fastn_unresolved::resolver::Output,
    // todo: how to resolve local symbols, eg inside a function / component
    _locals: Vec<fastn_unresolved::UR<fastn_unresolved::Argument, fastn_resolved::Argument>>,
) {
    let inner_name = if let fastn_unresolved::UR::UnResolved(name) = name {
        name
    } else {
        return;
    };

    // inner_name can be either:
    //   - foo: component defined in current module or import-exposed,
    //   - bar.foo: component defined in module bar,
    //   - bar#foo: component using the absolute path.
    //
    // our job is to verify that in case, such a symbol is defined, and return the full symbol name

    *name = fastn_unresolved::UR::Resolved(todo!());
}
