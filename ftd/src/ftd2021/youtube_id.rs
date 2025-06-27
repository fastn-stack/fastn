// source: https://docs.rs/rustube/0.3.4/src/rustube/id.rs.html#108-113 (MIT)

// todo: check patterns with regex debugger

pub static ID_PATTERNS: once_cell::sync::Lazy<Vec<regex::Regex>> =
    once_cell::sync::Lazy::new(|| {
        vec![
            // watch url    (i.e. https://youtube.com/watch?v=video_id)
            regex::Regex::new(
                r"^(https?://)?(www\.)?youtube.\w\w\w?/watch\?v=(?P<id>[a-zA-Z0-9_-]{11})(&.*)?$",
            )
            .unwrap(),
            // embed url    (i.e. https://youtube.com/embed/video_id)
            regex::Regex::new(
                r"^(https?://)?(www\.)?youtube.\w\w\w?/embed/(?P<id>[a-zA-Z0-9_-]{11})\\?(\?.*)?$",
            )
            .unwrap(),
            // share url    (i.e. https://youtu.be/video_id)
            regex::Regex::new(r"^(https?://)?youtu\.be/(?P<id>[a-zA-Z0-9_-]{11})$").unwrap(),
            // id           (i.e. video_id)
            regex::Regex::new("^(?P<id>[a-zA-Z0-9_-]{11})$").unwrap(),
        ]
    });

pub fn from_raw(raw: &str) -> Option<String> {
    ID_PATTERNS.iter().find_map(|pattern| {
        pattern.captures(raw).map(|c| {
            // will never panic because each pattern has an <id> defined
            let id = c.name("id").unwrap().as_str();
            format!("https://youtube.com/embed/{id}")
        })
    })
}
