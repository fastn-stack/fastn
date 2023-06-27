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
