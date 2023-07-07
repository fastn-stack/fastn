#[allow(dead_code)]
pub fn trim_all_lines(s: &str) -> String {
    use itertools::Itertools;

    s.split('\n').map(|v| v.trim()).join("\n")
}

pub(crate) fn update_reference_with_none(reference: &str) -> String {
    update_reference(reference, &None, &None, &None)
}

pub(crate) fn update_reference(
    reference: &str,
    component_definition_name: &Option<String>,
    loop_alias: &Option<String>,
    inherited_variable_name: &Option<String>,
) -> String {
    let name = reference.to_string();

    if let Some(component_definition_name) = component_definition_name {
        if let Some(alias) = name.strip_prefix(format!("{component_definition_name}.").as_str()) {
            return format!("{}.{alias}", fastn_js::LOCAL_VARIABLE_MAP);
        }
    }

    if let Some(loop_alias) = loop_alias {
        if let Some(alias) = name.strip_prefix(format!("{loop_alias}.").as_str()) {
            return format!("item.{alias}");
        } else if loop_alias.eq(&name) {
            return "item".to_string();
        }
    }

    if let Some(inherited_variable_name) = inherited_variable_name {
        if let Some(remaining) = name.strip_prefix("inherited.") {
            return format!("{inherited_variable_name}.{remaining}");
        }
    }

    if name.starts_with("inherited.") {
        return name;
    }

    if name.contains(ftd::interpreter::FTD_LOOP_COUNTER) {
        return "index".to_string();
    }

    format!("{}.{name}", fastn_js::GLOBAL_VARIABLE_MAP)
}
