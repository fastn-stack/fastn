pub fn is_kernel(s: &str) -> bool {
    [
        "ftd#text",
        "ftd#row",
        "ftd#column",
        "ftd#integer",
        "ftd#container",
    ]
    .contains(&s)
}

pub fn reference_to_js(s: &str) -> String {
    let (prefix, s) = get_prefix(s);

    let (mut p1, mut p2) = get_doc_name_and_remaining(s.as_str());
    p1 = fastn_js::utils::name_to_js_(p1.as_str());
    let mut prefix_attached = false;
    let mut wrapper_function = None;
    let is_asset_reference = p1.contains("assets");
    while let Some(ref remaining) = p2 {
        let (p21, p22) = get_doc_name_and_remaining(remaining);
        match p21.parse::<i64>() {
            Ok(num) if p22.is_none() => {
                p1 = format!("{}.get({})", p1, num);
                wrapper_function = Some("fastn_utils.getListItem");
            }
            Ok(num) if p22.is_some() && !prefix_attached && !is_asset_reference => {
                p1 = format!(
                    "fastn_utils.getListItem({}{}.get({}))",
                    prefix.map(|v| format!("{v}.")).unwrap_or_default(),
                    p1,
                    num
                );
                prefix_attached = true;
            }
            _ => {
                p1 = format!(
                    "{}.get(\"{}\")",
                    p1,
                    fastn_js::utils::name_to_js_(p21.as_str())
                );
                wrapper_function = None;
            }
        }
        p2 = p22;
    }
    if !prefix_attached {
        p1 = format!(
            "{}{p1}",
            prefix.map(|v| format!("{v}.")).unwrap_or_default()
        );
    }
    if let Some(func) = wrapper_function {
        return format!("{}({})", func, p1);
    }
    p1
}

pub fn clone_to_js(s: &str) -> String {
    format!("fastn_utils.clone({})", reference_to_js(s))
}

pub(crate) fn get_doc_name_and_remaining(s: &str) -> (String, Option<String>) {
    let mut part1 = "".to_string();
    let mut pattern_to_split_at = s.to_string();
    if let Some((p1, p2)) = s.split_once('#') {
        part1 = format!("{}#", p1);
        pattern_to_split_at = p2.to_string();
    }
    if let Some((p1, p2)) = pattern_to_split_at.split_once('.') {
        (format!("{}{}", part1, p1), Some(p2.to_string()))
    } else {
        (s.to_string(), None)
    }
}

fn get_prefix(s: &str) -> (Option<&str>, String) {
    let mut s = s.to_string();
    let prefix = if let Some(prefix) =
        s.strip_prefix(format!("{}.", fastn_js::GLOBAL_VARIABLE_MAP).as_str())
    {
        s = prefix.to_string();
        Some(fastn_js::GLOBAL_VARIABLE_MAP)
    } else if let Some(prefix) =
        s.strip_prefix(format!("{}.", fastn_js::LOCAL_VARIABLE_MAP).as_str())
    {
        s = prefix.to_string();
        Some(fastn_js::LOCAL_VARIABLE_MAP)
    } else if let Some(prefix) = s.strip_prefix("ftd.").or(s.strip_prefix("ftd#")) {
        s = prefix.to_string();
        Some("ftd")
    } else if let Some(prefix) = s.strip_prefix("fastn_utils.") {
        s = prefix.to_string();
        Some("fastn_utils")
    } else {
        None
    };
    (prefix, s)
}

pub(crate) fn is_local_variable_map_prefix(s: &str) -> bool {
    fastn_js::utils::get_prefix(s)
        .0
        .map(|v| v.eq(fastn_js::LOCAL_VARIABLE_MAP))
        .unwrap_or_default()
}

pub fn name_to_js(s: &str) -> String {
    let (prefix, s) = get_prefix(s);
    format!(
        "{}{}",
        prefix.map(|v| format!("{v}.")).unwrap_or_default(),
        name_to_js_(s.as_str())
    )
}

pub fn name_to_js_(s: &str) -> String {
    let mut s = s.to_string();
    //todo: remove this
    if s.as_bytes()[0].is_ascii_digit() {
        s = format!("_{}", s);
    }
    s.replace('#', "__")
        .replace('-', "_")
        .replace(':', "___")
        .replace(',', "$")
        .replace("\\\\", "/")
        .replace('\\', "/")
        .replace(['/', '.'], "_")
}

pub fn trim_brackets(s: &str) -> String {
    if s.starts_with('(') && s.ends_with(')') {
        return s[1..s.len() - 1].to_string();
    }
    s.to_string()
}

pub(crate) fn kebab_to_snake_case(s: &str) -> String {
    s.replace('-', "_")
}
