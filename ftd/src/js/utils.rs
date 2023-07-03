#[allow(dead_code)]
pub fn trim_all_lines(s: &str) -> String {
    use itertools::Itertools;

    s.split('\n').map(|v| v.trim()).join("\n")
}

pub(crate) fn update_reference(
    reference: &str,
    component_definition_name: &Option<String>,
    loop_alias: &Option<String>,
) -> String {
    let mut name = reference
        .trim_start_matches(
            format!("{}.", component_definition_name.clone().unwrap_or_default()).as_str(),
        )
        .to_string();
    if let Some(loop_alias) = loop_alias {
        if let Some(alias) = name.strip_prefix(format!("{loop_alias}.").as_str()) {
            name = format!("item.{alias}");
        } else if loop_alias.eq(&name) {
            name = "item".to_string()
        }
    }

    if name.contains(ftd::interpreter::FTD_LOOP_COUNTER) {
        name = "index".to_string()
    }

    name
}
