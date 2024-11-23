pub fn resolve(
    name: &mut fastn_unresolved::UR<fastn_unresolved::Identifier, fastn_unresolved::Identifier>,
    _input: fastn_unresolved::resolver::Input<'_>,
    _output: &mut fastn_unresolved::resolver::Output,
) {
    let inner_name = if let fastn_unresolved::UR::UnResolved(name) = name {
        name
    } else {
        return;
    };

    // verify name is good

    *name = fastn_unresolved::UR::Resolved(inner_name.clone());
}
