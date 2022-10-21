pub fn trim_all_lines(s: &str) -> String {
    use itertools::Itertools;

    s.split('\n').into_iter().map(|v| v.trim()).join("\n")
}
