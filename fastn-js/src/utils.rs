pub fn is_kernel(s: &str) -> bool {
    ["ftd#text", "ftd#row", "ftd#column", "ftd#integer"].contains(&s)
}

pub fn reference_to_js(s: &str) -> String {
    let (mut p1, mut p2) = get_doc_name_and_remaining(s);
    p1 = fastn_js::utils::name_to_js(p1.as_str());
    while let Some(remaining) = p2 {
        let (p21, p22) = get_doc_name_and_remaining(remaining.as_str());
        p1 = format!("{}.get(\"{}\")", p1, p21);
        p2 = p22;
    }
    p1
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

pub fn name_to_js(s: &str) -> String {
    let mut s = s.to_string();
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

pub fn kebab_to_snake_case(s: &str) -> String {
    s.replace('-', "_")
}
