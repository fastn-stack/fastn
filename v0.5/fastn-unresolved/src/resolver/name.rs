pub fn resolve<'a>(
    name: &mut fastn_unresolved::UR<fastn_unresolved::Identifier, fastn_unresolved::Identifier>,
    _input: &fastn_unresolved::resolver::Input<'a>,
    _output: &mut fastn_unresolved::resolver::Output,
) -> Option<&'a fastn_resolved::ComponentDefinition> {
    let inner_name = if let fastn_unresolved::UR::UnResolved(name) = name {
        name
    } else {
        return None;
    };

    // verify name is good

    *name = fastn_unresolved::UR::Resolved(inner_name.clone());

    None
}
