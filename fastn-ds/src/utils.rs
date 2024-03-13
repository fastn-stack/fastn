pub fn ignore_headers() -> Vec<&'static str> {
    vec!["host", "x-forwarded-ssl"]
}

// https://stackoverflow.com/questions/71985357/whats-the-best-way-to-write-a-custom-format-macro
#[macro_export]
macro_rules! warning {
    ($($t:tt)*) => {{
        use colored::Colorize;
        let msg = format!($($t)*);
        if fastn_observer::is_traced() {
            tracing::warn!(msg);
        } else {
            eprintln!("WARN: {}", msg.yellow());
        }
        msg
    }};
}
