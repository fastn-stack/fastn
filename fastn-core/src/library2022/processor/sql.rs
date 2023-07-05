fn extract_arguments(query: &str) -> ftd::interpreter::Result<(String, Vec<String>)> {
    let chars: Vec<char> = query.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut found_eq = false;
    let mut escaped = false;
    let mut args: Vec<String> = Vec::new();
    let mut output_query = String::new();

    while i < len {
        if chars[i] == '=' {
            found_eq = true;
        }

        if chars[i] == '\\' {
            escaped = true;

            let mut escape_count = 0;
            
            while i < len && chars[i] == '\\' {
                escape_count += 1;
                i += 1;
            }

            if escape_count % 2 == 0 {
                output_query += &"\\".repeat(escape_count);
                escaped = false;
            }
        }

        if found_eq && chars[i] == '$' && !escaped {
            let mut arg = String::new();
            i += 1;

            while i < len && chars[i] != ' ' {
                arg.push(chars[i]);
                i += 1;
            }

            if !arg.is_empty() {
                let index = args.iter().position(|x| x == &arg);

                let gap = if i < len { " " } else { "" };

                if let Some(idx) = index {
                    output_query += &format!("${}{}", idx + 1, gap);
                } else {
                    args.push(arg.clone());
                    output_query += &format!("${}{}", args.len(), gap);
                }
            }

            found_eq = false;
        } else {
            if escaped {
                output_query += "\\";
                escaped = false;
            }

            output_query.push(chars[i]);
        }

        i += 1;
    }

    Ok((output_query, args))
}

#[cfg(test)]
mod test {
    #[track_caller]
    fn e(i: &str, o: &str, a: Vec<&str>) {
        let (query, arguments) = super::extract_arguments(i).unwrap();
        assert_eq!(query, o);
        assert_eq!(arguments, a);
    }

    #[test]
    fn extract_arguments() {
        e("hello", "hello", vec![]);
        e(
            "SELECT * FROM test where name = $name",
            "SELECT * FROM test where name = $1",
            vec!["name"],
        );
        e(
            "SELECT * FROM test where name = $name and full_name = $full_name",
            "SELECT * FROM test where name = $1 and full_name = $2",
            vec!["name", "full_name"],
        );
        e(
            r#"SELECT * FROM test where name = \$name and full_name = $full_name"#,
            r#"SELECT * FROM test where name = \$name and full_name = $1"#,
            vec!["full_name"],
        );
        e(
            r#"SELECT * FROM test where name = \\$name and full_name = $full_name"#,
            r#"SELECT * FROM test where name = \\$1 and full_name = $2"#,
            vec!["name", "full_name"],
        );
        e(
            "SELECT * FROM test where name = $name and full_name = $name",
            "SELECT * FROM test where name = $1 and full_name = $1",
            vec!["name"],
        );
    }
}
