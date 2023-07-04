pub(crate) fn extract_arguments(query: &str) -> ftd::interpreter::Result<(String, Vec<String>)> {
    let chars: Vec<char> = query.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut found_eq = false;
    let mut args: Vec<String> = Vec::new();
    let mut output_query = String::new();

    while i < len {
        let current_char = chars[i];

        if current_char == '=' {
            found_eq = true;
        }

        if found_eq && current_char == '$' && chars[i - 1] != '\\' {
            let mut arg = String::new();
            i += 1;

            while i < len && chars[i] != ' ' {
                arg.push(chars[i]);
                i += 1;
            }

            let gap = if i < len { " " } else { "" };

            if !arg.is_empty() {
                let index = args.iter().position(|x| *x == arg);

                match index {
                    Some(idx) => {
                        output_query.push_str(&format!("${}{}", idx + 1, gap));
                    }
                    None => {
                        args.push(arg.clone());
                        output_query.push_str(&format!("${}{}", args.len(), gap));
                    }
                }
            }
        } else {
            output_query.push(current_char);
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
