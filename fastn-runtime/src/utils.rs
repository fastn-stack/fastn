use fastn_resolved_to_js::extensions::*;

#[allow(dead_code)]
pub fn trim_all_lines(s: &str) -> String {
    use itertools::Itertools;

    s.split('\n').map(|v| v.trim()).join("\n")
}

pub fn get_js_html(external_js: &[String]) -> String {
    let mut result = "".to_string();
    for js in external_js {
        if let Some((js, tags)) = js.rsplit_once(':') {
            result = format!("{}<script src=\"{}\" {}></script>", result, js, tags);
        } else {
            result = format!("{}<script src=\"{}\"></script>", result, js);
        }
    }
    result
}

pub fn get_css_html(external_css: &[String]) -> String {
    let mut result = "".to_string();
    for css in external_css {
        result = format!("{}<link rel=\"stylesheet\" href=\"{}\">", result, css);
    }
    result
}

pub(crate) fn get_rive_event(
    events: &[fastn_resolved::Event],
    doc: &dyn fastn_resolved::tdoc::TDoc,
    rdata: &fastn_resolved_to_js::ResolverData,
    element_name: &str,
) -> String {
    let mut events_map: fastn_resolved_to_js::VecMap<(&String, &fastn_resolved::FunctionCall)> =
        fastn_resolved_to_js::VecMap::new();
    for event in events.iter() {
        let (event_name, input, action) = match &event.name {
            fastn_resolved::EventName::RivePlay(timeline) => ("onPlay", timeline, &event.action),
            fastn_resolved::EventName::RivePause(timeline) => ("onPause", timeline, &event.action),
            fastn_resolved::EventName::RiveStateChange(state) => {
                ("onStateChange", state, &event.action)
            }
            _ => continue,
        };
        events_map.insert(event_name.to_string(), (input, action));
    }
    let mut events_vec = vec![];
    for (on, actions) in events_map.value {
        let mut actions_vec = vec![];
        for (input, action) in actions {
            let action =
                fastn_resolved_to_js::utils::function_call_to_js_formula(action, doc, rdata)
                    .formula_value_to_js(&Some(element_name.to_string()));
            actions_vec.push(format!(
                indoc::indoc! {"
                      if (input === \"{input}\") {{
                        let action = {action};
                        action();
                      }}
                "},
                input = input,
                action = action
            ));
        }

        events_vec.push(format!(
            indoc::indoc! {"
                    {on}: (event) => {{
                        const inputs = event.data;
                        inputs.forEach((input) => {{
                          {actions_vec}
                        }});
                    }},
                "},
            on = on,
            actions_vec = actions_vec.join("\n")
        ));
    }
    events_vec.join("\n")
}

pub fn get_external_scripts(has_rive_components: bool) -> Vec<String> {
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

pub(crate) fn update_reference(
    reference: &str,
    rdata: &fastn_resolved_to_js::ResolverData,
) -> String {
    let name = reference.to_string();

    if fastn_builtins::constants::FTD_SPECIAL_VALUE
        .trim_start_matches('$')
        .eq(reference)
    {
        let component_name = rdata.component_name.clone().unwrap();
        return format!("fastn_utils.getNodeValue({component_name})");
    }

    if fastn_builtins::constants::FTD_SPECIAL_CHECKED
        .trim_start_matches('$')
        .eq(reference)
    {
        let component_name = rdata.component_name.clone().unwrap();
        return format!("fastn_utils.getNodeCheckedState({component_name})");
    }

    if let Some(component_definition_name) = rdata.component_definition_name {
        if let Some(alias) = name.strip_prefix(format!("{component_definition_name}.").as_str()) {
            return format!("{}.{alias}", fastn_js::LOCAL_VARIABLE_MAP);
        }
    }

    if let Some(record_definition_name) = rdata.record_definition_name {
        if let Some(alias) = name.strip_prefix(format!("{record_definition_name}.").as_str()) {
            return format!("{}.{alias}", fastn_js::LOCAL_RECORD_MAP);
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

    if let Some(loop_counter_alias) = rdata.loop_counter_alias {
        if let Some(ref doc_name) = rdata.doc_name {
            let resolved_alias = fastn_resolved_to_js::utils::resolve_name(
                loop_counter_alias,
                doc_name.as_str(),
                &fastn_builtins::default_aliases(),
            );

            if name.eq(resolved_alias.as_str()) {
                return "index".to_string();
            }
        }
    }

    if name.contains(fastn_builtins::constants::FTD_LOOP_COUNTER) {
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
    properties: &[fastn_resolved::Property],
) -> Option<fastn_resolved_to_js::Value> {
    use fastn_resolved_to_js::extensions::PropertyValueExt;
    if properties.is_empty() {
        return None;
    }

    if properties.len() == 1 {
        let property = properties.first().unwrap();
        if property.condition.is_none() {
            return Some(property.value.to_value());
        }
    }

    Some(fastn_resolved_to_js::Value::ConditionalFormula(
        properties.to_owned(),
    ))
}

pub(crate) fn function_call_to_js_formula(
    function_call: &fastn_resolved::FunctionCall,
    doc: &dyn fastn_resolved::tdoc::TDoc,
    rdata: &fastn_resolved_to_js::ResolverData,
) -> fastn_js::Formula {
    let mut deps = vec![];
    for property_value in function_call.values.values() {
        deps.extend(property_value.get_deps(rdata));
    }

    fastn_js::Formula {
        deps,
        type_: fastn_js::FormulaType::FunctionCall(function_call.to_js_function(doc, rdata)),
    }
}

pub(crate) fn is_ui_argument(
    component_arguments: &[fastn_resolved::Argument],
    remaining: &str,
) -> bool {
    component_arguments
        .iter()
        .any(|a| a.name.eq(remaining) && a.kind.is_ui())
}

pub(crate) fn is_module_argument(
    component_arguments: &[fastn_resolved::Argument],
    remaining: &str,
) -> Option<String> {
    let (module_name, component_name) = remaining.split_once('.')?;
    component_arguments.iter().find_map(|v| {
        if v.name.eq(module_name) && v.kind.is_module() {
            let module = v
                .value
                .as_ref()
                .and_then(|v| v.value_optional())
                .and_then(|v| v.module_name_optional())?;
            Some(format!("{module}#{component_name}"))
        } else {
            None
        }
    })
}

/// Retrieves `fastn_js::SetPropertyValue` for user provided component properties only not the
/// arguments with default.
///
/// This function attempts to retrieve component or web component arguments based on the provided
/// component name. It then filters out valid arguments whose value is provided by user. The
/// function returns argument name and the corresponding `fastn_js::SetPropertyValue` as a vector
/// of tuples.
///
/// # Arguments
///
/// * `doc` - A reference to the TDoc object containing the document's data.
/// * `component_name` - The name of the component or web component to retrieve arguments for.
/// * `component_properties` - The list of component properties to match against arguments.
/// * `line_number` - The line number associated with the component.
///
/// # Returns
///
/// An `Option` containing a vector of tuples where the first element is the argument name and the
/// second element is the corresponding set property value. Returns `None` if any retrieval or
/// conversion operation fails.
pub(crate) fn get_set_property_values_for_provided_component_properties(
    doc: &dyn fastn_resolved::tdoc::TDoc,
    rdata: &fastn_resolved_to_js::ResolverData,
    component_name: &str,
    component_properties: &[fastn_resolved::Property],
    has_rive_components: &mut bool,
) -> Option<Vec<(String, fastn_js::SetPropertyValue, bool)>> {
    use itertools::Itertools;

    // Attempt to retrieve component or web component arguments
    doc.get_opt_component(component_name)
        .map(|v| v.arguments.clone())
        .or(doc
            .get_opt_web_component(component_name)
            .map(|v| v.arguments.clone()))
        .map(|arguments| {
            // Collect valid arguments matching the provided properties and their set property values
            arguments
                .iter()
                .filter(|argument| !argument.kind.is_kwargs())
                .filter_map(|v| {
                    v.get_optional_value(component_properties).map(|val| {
                        (
                            v.name.to_string(),
                            val.to_set_property_value_with_ui(
                                doc,
                                rdata,
                                has_rive_components,
                                false,
                            ),
                            v.mutable,
                        )
                    })
                })
                .collect_vec()
        })
}

pub(crate) fn get_doc_name_and_remaining(s: &str) -> (String, Option<String>) {
    let mut part1 = "".to_string();
    let mut pattern_to_split_at = s.to_string();
    if let Some((p1, p2)) = s.split_once('#') {
        part1 = format!("{}#", p1);
        pattern_to_split_at = p2.to_string();
    }
    if pattern_to_split_at.contains('.') {
        let (p1, p2) = split(pattern_to_split_at.as_str(), ".").unwrap();
        (format!("{}{}", part1, p1), Some(p2))
    } else {
        (s.to_string(), None)
    }
}

pub fn split(name: &str, split_at: &str) -> Option<(String, String)> {
    if !name.contains(split_at) {
        return None;
    }
    let mut part = name.splitn(2, split_at);
    let part_1 = part.next().unwrap().trim();
    let part_2 = part.next().unwrap().trim();
    Some((part_1.to_string(), part_2.to_string()))
}

pub fn get_doc_name_and_thing_name_and_remaining(
    s: &str,
    doc_id: &str,
) -> (String, String, Option<String>) {
    let (doc_name, remaining) = get_doc_name_and_remaining(s);
    if let Some((doc_name, thing_name)) = doc_name.split_once('#') {
        (doc_name.to_string(), thing_name.to_string(), remaining)
    } else {
        (doc_id.to_string(), doc_name, remaining)
    }
}

pub fn get_children_properties_from_properties(
    properties: &[fastn_resolved::Property],
) -> Vec<fastn_resolved::Property> {
    use itertools::Itertools;

    properties
        .iter()
        .filter_map(|v| {
            if v.value.kind().inner_list().is_subsection_ui() {
                Some(v.to_owned())
            } else {
                None
            }
        })
        .collect_vec()
}

pub fn resolve_name(name: &str, doc_name: &str, aliases: &fastn_builtins::Map<String>) -> String {
    let name = name
        .trim_start_matches(fastn_resolved_to_js::CLONE)
        .trim_start_matches(fastn_resolved_to_js::REFERENCE)
        .to_string();

    if name.contains('#') {
        return name;
    }

    let doc_name = doc_name.trim_end_matches('/');
    match fastn_resolved_to_js::utils::split_module(name.as_str()) {
        (Some(m), v, None) => match aliases.get(m) {
            Some(m) => format!("{}#{}", m, v),
            None => format!("{}#{}.{}", doc_name, m, v),
        },
        (Some(m), v, Some(c)) => match aliases.get(m) {
            Some(m) => format!("{}#{}.{}", m, v, c),
            None => format!("{}#{}.{}.{}", doc_name, m, v, c),
        },
        (None, v, None) => format!("{}#{}", doc_name, v),
        _ => unimplemented!(),
    }
}

pub fn split_module(id: &str) -> (Option<&str>, &str, Option<&str>) {
    match id.split_once('.') {
        Some((p1, p2)) => match p2.split_once('.') {
            Some((p21, p22)) => (Some(p1), p21, Some(p22)),
            None => (Some(p1), p2, None),
        },
        None => (None, id, None),
    }
}

pub(crate) fn find_properties_by_source_without_default(
    sources: &[fastn_resolved::PropertySource],
    properties: &[fastn_resolved::Property],
) -> Vec<fastn_resolved::Property> {
    use itertools::Itertools;

    properties
        .iter()
        .filter(|v| sources.iter().any(|s| v.source.is_equal(s)))
        .map(ToOwned::to_owned)
        .collect_vec()
}
