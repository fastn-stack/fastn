pub(crate) fn is_kernel(s: &str) -> bool {
    ["ftd#text", "ftd#row", "ftd#column"].contains(&s)
}

#[cfg(test)]
pub fn trim_all_lines(s: &str) -> String {
    use itertools::Itertools;

    s.split('\n').map(|v| v.trim()).join("\n")
}
