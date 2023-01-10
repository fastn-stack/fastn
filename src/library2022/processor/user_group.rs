pub fn process<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    use itertools::Itertools;
    let g = config
        .package
        .groups
        .iter()
        .map(|(_, g)| g.to_group_compat())
        .collect_vec();
    doc.from_json(&g, &kind, value.line_number())
}
