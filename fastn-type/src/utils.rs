pub(crate) fn update_reference(reference: &str, rdata: &fastn_type::ResolverData) -> String {
    let name = reference.to_string();

    if fastn_type::FTD_SPECIAL_VALUE
        .trim_start_matches('$')
        .eq(reference)
    {
        let component_name = rdata.component_name.clone().unwrap();
        return format!("fastn_utils.getNodeValue({component_name})");
    }

    if fastn_type::FTD_SPECIAL_CHECKED
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
            let resolved_alias = fastn_type::utils::resolve_name(
                loop_counter_alias,
                doc_name.as_str(),
                &fastn_type::default::default_aliases(),
            );

            if name.eq(resolved_alias.as_str()) {
                return "index".to_string();
            }
        }
    }

    if name.contains(fastn_type::FTD_LOOP_COUNTER) {
        return "index".to_string();
    }

    if is_ftd_thing(name.as_str()) {
        return name.replace("ftd#", "ftd.");
    }

    format!("{}.{name}", fastn_js::GLOBAL_VARIABLE_MAP)
}

pub(crate) fn function_call_to_js_formula(
    function_call: &fastn_type::FunctionCall,
    doc: &fastn_type::TDoc,
    rdata: &fastn_type::ResolverData,
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

fn is_ftd_thing(name: &str) -> bool {
    name.starts_with("ftd#") || name.starts_with("ftd.")
}

pub fn resolve_name(name: &str, doc_name: &str, aliases: &fastn_type::Map<String>) -> String {
    let name = name
        .trim_start_matches(fastn_type::CLONE)
        .trim_start_matches(fastn_type::REFERENCE)
        .to_string();

    if name.contains('#') {
        return name;
    }

    let doc_name = doc_name.trim_end_matches('/');
    match fastn_type::utils::split_module(name.as_str()) {
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
