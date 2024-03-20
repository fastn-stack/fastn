pub fn ignore_headers() -> Vec<&'static str> {
    vec!["host", "x-forwarded-ssl"]
}
