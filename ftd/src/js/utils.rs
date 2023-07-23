#[allow(dead_code)]
pub fn trim_all_lines(s: &str) -> String {
    use itertools::Itertools;

    s.split('\n').map(|v| v.trim()).join("\n")
}

pub(crate) fn get_external_scripts(has_rive_components: bool) -> Vec<String> {
    let mut scripts = vec![];
    if has_rive_components {
        scripts.push(
            "<script src=\"https://unpkg.com/@rive-app/canvas@1.0.98\"></script>".to_string(),
        );
    }
    scripts
}

pub(crate) fn to_key(key: &str) -> String {
    match key {
        "ctrl" => "Control",
        "alt" => "Alt",
        "shift" => "Shift",
        "up" => "ArrowUp",
        "down" => "ArrowDown",
        "right" => "ArrowRight",
        "left" => "ArrowLeft",
        "esc" => "Escape",
        "dash" => "-",
        "space" => " ",
        t => t,
    }
    .to_string()
}

pub(crate) fn update_reference_with_none(reference: &str) -> String {
    update_reference(reference, &ftd::js::ResolverData::none())
}

pub(crate) fn update_reference(reference: &str, rdata: &ftd::js::ResolverData) -> String {
    let name = reference.to_string();

    if let Some(component_definition_name) = rdata.component_definition_name {
        if let Some(alias) = name.strip_prefix(format!("{component_definition_name}.").as_str()) {
            return format!("{}.{alias}", fastn_js::LOCAL_VARIABLE_MAP);
        }
    }

    if let Some(loop_alias) = rdata.loop_alias {
        if let Some(alias) = name.strip_prefix(format!("{loop_alias}.").as_str()) {
            return format!("item.{alias}");
        } else if loop_alias.eq(&name) {
            return "item".to_string();
        }
    }

    if let Some(remaining) = name.strip_prefix("inherited.") {
        return format!("{}.{remaining}", rdata.inherited_variable_name);
    }

    if name.contains(ftd::interpreter::FTD_LOOP_COUNTER) {
        return "index".to_string();
    }

    if is_ftd_thing(name.as_str()) {
        return name.replace("ftd#", "ftd.");
    }

    format!("{}.{name}", fastn_js::GLOBAL_VARIABLE_MAP)
}

fn is_ftd_thing(name: &str) -> bool {
    name.starts_with("ftd#") || name.starts_with("ftd.")
}

pub(crate) fn get_js_value_from_properties(
    properties: &[ftd::interpreter::Property],
) -> Option<ftd::js::Value> {
    if properties.is_empty() {
        return None;
    }

    if properties.len() == 1 {
        let property = properties.first().unwrap();
        if property.condition.is_none() {
            return Some(property.value.to_value());
        }
    }

    Some(ftd::js::Value::ConditionalFormula(properties.to_owned()))
}
