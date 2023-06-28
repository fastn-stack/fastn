pub fn is_kernel(s: &str) -> bool {
    ["ftd#text", "ftd#row", "ftd#column"].contains(&s)
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
