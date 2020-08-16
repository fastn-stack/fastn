use std::collections::HashMap;
use std::slice::Iter;

fn remove_comments<'a>(lines: &Vec<&str>) -> Vec<String> {
    let mut new_lines = vec![];
    for line in lines {
        if line.trim().starts_with(";") {
            continue;
        }
        new_lines.push(line.to_string())
    }
    new_lines
}

fn parser(txt_lines: &mut Vec<String>) -> Result<serde_json::Value, String> {
    let mut sections: Vec<serde_json::Value> = vec![];
    let mut section: HashMap<String, serde_json::Value> = HashMap::new();
    while txt_lines.len() > 0 {
        let mut line = txt_lines.pop().unwrap();
        // println!("Line: {:?}", line);
        if line.starts_with("--") {
            if !section.is_empty() {
                sections.push(serde_json::to_value(section)
                    .map_err(|_| "error_occurred".to_string())?)
            }
            section = HashMap::new();
            let section_header: Vec<&str> = line.split(":").collect();
            let section_name_split: Vec<&str> = section_header
                .get(0)
                .expect("invalid_section_header")
                .split_whitespace()
                .collect();
            let section_name = section_name_split.get(1).expect("section_name_error");
            let section_caption = section_header.get(1).unwrap_or(&"").trim();
            section.insert(
                "section".to_string(),
                serde_json::to_value(section_name).map_err(|_| "error".to_string())?,
            );
            section.insert(
                "caption".to_string(),
                serde_json::to_value(section_caption).map_err(|_| "error".to_string())?,
            );
            continue;
        }
        // handle key values
        if line.contains(":") {
            let key_value: Vec<&str> = line.split(":").collect();
            let key = key_value.get(0).ok_or("key_error")?.trim();
            let value = key_value.get(1).ok_or("value_error")?.trim();
            section.insert(
                key.into(),
                serde_json::to_value(value).map_err(|_| "error".to_string())?,
            );
            continue;
        }

        // handle object and list
        if line.starts_with("---") {
            continue;
        }

        // handle body
        let mut body = "".to_string();
        while !line.is_empty() && !(line.starts_with("--") || line.starts_with("---")) {
            body += line.as_ref();
            line = txt_lines.pop().unwrap_or("".to_string());
        }
        if !line.is_empty() {
            txt_lines.push(line);
        }
        if !body.is_empty() {
            section.insert(
                "body".to_string(),
                serde_json::to_value(body).map_err(|_| "".to_string())?,
            );
        }
        // println!("{:?}", section);
    }
    if !section.is_empty() {
        sections.push(serde_json::to_value(section).map_err(|_| "".to_string())?)
    }
    Ok(serde_json::to_value(sections).map_err(|_| "error_occurred".to_string())?)
}

fn parse(txt: &str) {
    let mut data: Vec<&str> = txt.split("\n").collect();
    data.reverse();
    let mut lines: Vec<String> = remove_comments(&data);
    println!("{:?}", parser(&mut lines));
}

fn main() {
    let raw_str = r#"; this is a comment
-- amitu/table: Some table
columns: l | c | r

some body

"#;
    parse(raw_str);
    let raw_str_1 = r#"; this is a comment
-- amitu/table: Some table
key: value

columns: l | c | r

some body
"#;
}
