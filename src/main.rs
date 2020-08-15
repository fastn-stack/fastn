use std::collections::HashMap;

fn remove_comments<'a>(lines: Vec<&str>) -> Vec<&str> {
    let mut new_lines = vec![];
    for line in lines {
        if line.trim().starts_with(";") {
            continue;
        }
        new_lines.push(line)
    }
    new_lines
}

fn parser(lines: Vec<&str>) -> serde_json::Value {
    let mut sections: Vec<serde_json::Value> = vec![];
    let mut section: HashMap<String, serde_json::Value>;
    println!("{:#?}", sections);
    for line in lines {
        if line.starts_with("--") {
            println!("section starts")
        }
        if line.starts_with("\\--") {
            println!("section starts")
        }
        if line.starts_with("---") {
            println!("row starts")
        }
        if line.starts_with("\\---") {
            println!("row starts")
        }
    }
    serde_json::to_value(sections).unwrap()
}

fn parse(txt: &str) {
    let lines: Vec<&str> = remove_comments(txt.split("\n").collect());
    parser(lines);
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
