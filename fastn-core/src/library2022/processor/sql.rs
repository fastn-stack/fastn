use crate::library2022::processor::sqlite::result_to_value;

pub async fn process(
    value: ftd_ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::RequestConfig,
    is_query: bool,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let (headers, query) = super::sqlite::get_p1_data(
        if is_query { "sql-query" } else { "sql-execute" },
        &value,
        doc.name,
    )?;
    let db = match headers.get_optional_string_by_key("db$", doc.name, value.line_number())? {
        Some(db) => db,
        None => match config.config.ds.env("FASTN_DB_URL").await {
            Ok(db_url) => db_url,
            Err(_) => config
                .config
                .ds
                .env("DATABASE_URL")
                .await
                .unwrap_or_else(|_| "fastn.sqlite".to_string()),
        },
    };

    let (query, params) = crate::library2022::processor::sqlite::extract_named_parameters(
        query.as_str(),
        doc,
        headers,
        value.line_number(),
    )?;

    let ds = &config.config.ds;

    let res = match if is_query {
        ds.sql_query(db.as_str(), query.as_str(), params).await
    } else {
        ds.sql_execute(db.as_str(), query.as_str(), params).await
    } {
        Ok(v) => v,
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("Error executing query: {e:?}"),
                doc.name,
                value.line_number(),
            )
        }
    };

    result_to_value(res, kind, doc, &value)
}

const BACKSLASH: char = '\\';
const SPECIAL_CHARS: [char; 9] = [BACKSLASH, '$', '/', ':', '"', ',', '\'', ';', ' '];
const SQLITE_SUB: char = '?';
const POSTGRES_SUB: char = '$';

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
    #[error("Invalid query, quote left open")]
    QuoteOpen,
}

#[allow(dead_code)]
pub(crate) fn extract_arguments(
    query: &str,
    sub: char,
) -> Result<(String, Vec<String>), QueryError> {
    let chars: Vec<char> = query.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut quote: Option<char> = None;
    let mut quote_open = false;
    let mut escaped = false;
    let mut args: Vec<String> = Vec::new();
    let mut output_query = String::new();

    while i < len {
        if chars[i] == BACKSLASH {
            escaped = true;
            let mut escape_count = 0;

            while i < len && chars[i] == BACKSLASH {
                escape_count += 1;
                i += 1;
            }

            if escape_count % 2 == 0 {
                output_query += &BACKSLASH.to_string().repeat(escape_count);
                escaped = false;
            }
        }

        if chars[i] == '"' && !escaped {
            if quote_open {
                if Some(chars[i]) == quote {
                    quote_open = false;
                    quote = None;
                }
            } else {
                quote_open = true;
                quote = Some(chars[i]);
            }
        }

        if chars[i] == '$' && !escaped && !quote_open {
            let mut arg = String::new();
            i += 1;

            while i < len {
                if SPECIAL_CHARS.contains(&chars[i]) {
                    i -= 1;
                    break;
                } else {
                    arg.push(chars[i]);
                    i += 1;
                }
            }

            if !arg.is_empty() {
                if let Some(index) = args.iter().position(|x| x == &arg) {
                    output_query += &format!("{sub}{}", index + 1);
                } else {
                    args.push(arg.clone());
                    output_query += &format!("{sub}{}", args.len());
                }
            }
        } else {
            if escaped {
                output_query += &BACKSLASH.to_string();
                escaped = false;
            }
            output_query.push(chars[i]);
        }

        i += 1;
    }

    if quote_open {
        return Err(QueryError::QuoteOpen);
    }

    Ok((output_query, args))
}

#[cfg(test)]
mod test {
    #[track_caller]
    fn e(i: &str, o: &str, a: Vec<&str>) {
        let (query, arguments) = super::extract_arguments(i, super::POSTGRES_SUB).unwrap();
        assert_eq!(query, o);
        assert_eq!(arguments, a);
    }

    #[track_caller]
    fn f(i: &str, o: &str, a: Vec<&str>) {
        let (query, arguments) = super::extract_arguments(i, super::SQLITE_SUB).unwrap();
        assert_eq!(query, o);
        assert_eq!(arguments, a);
    }

    #[test]
    fn extract_arguments() {
        e("SELECT $val::FLOAT8;", "SELECT $1::FLOAT8;", vec!["val"]);
        e(
            "SELECT * FROM test where name = $name;",
            "SELECT * FROM test where name = $1;",
            vec!["name"],
        );
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
            r"SELECT * FROM test where name = \$name and full_name = $full_name",
            r"SELECT * FROM test where name = \$name and full_name = $1",
            vec!["full_name"],
        );
        e(
            r"SELECT * FROM test where name = \\$name and full_name = $full_name",
            r"SELECT * FROM test where name = \\$1 and full_name = $2",
            vec!["name", "full_name"],
        );
        e(
            "SELECT * FROM test where name = $name and full_name = $name",
            "SELECT * FROM test where name = $1 and full_name = $1",
            vec!["name"],
        );
        e(
            "SELECT * FROM test where name = \"$name\" and full_name = $name",
            "SELECT * FROM test where name = \"$name\" and full_name = $1",
            vec!["name"],
        );
        e(
            "SELECT * FROM test where name = \"'$name'\" and full_name = $name",
            "SELECT * FROM test where name = \"'$name'\" and full_name = $1",
            vec!["name"],
        );
        e(
            r#"SELECT * FROM test where name = \"$name\" and full_name = $name"#,
            r#"SELECT * FROM test where name = \"$1\" and full_name = $1"#,
            vec!["name"],
        );

        f("SELECT $val::FLOAT8;", "SELECT ?1::FLOAT8;", vec!["val"]);
        f(
            "SELECT * FROM test where name = $name;",
            "SELECT * FROM test where name = ?1;",
            vec!["name"],
        );
        f("hello", "hello", vec![]);
        f(
            "SELECT * FROM test where name = $name",
            "SELECT * FROM test where name = ?1",
            vec!["name"],
        );
        f(
            "SELECT * FROM test where name = $name and full_name = $full_name",
            "SELECT * FROM test where name = ?1 and full_name = ?2",
            vec!["name", "full_name"],
        );
        f(
            r"SELECT * FROM test where name = \$name and full_name = $full_name",
            r"SELECT * FROM test where name = \$name and full_name = ?1",
            vec!["full_name"],
        );
        f(
            r"SELECT * FROM test where name = \\$name and full_name = $full_name",
            r"SELECT * FROM test where name = \\?1 and full_name = ?2",
            vec!["name", "full_name"],
        );
        f(
            "SELECT * FROM test where name = $name and full_name = $name",
            "SELECT * FROM test where name = ?1 and full_name = ?1",
            vec!["name"],
        );
        f(
            "SELECT * FROM test where name = \"$name\" and full_name = $name",
            "SELECT * FROM test where name = \"$name\" and full_name = ?1",
            vec!["name"],
        );
        f(
            "SELECT * FROM test where name = \"'$name'\" and full_name = $name",
            "SELECT * FROM test where name = \"'$name'\" and full_name = ?1",
            vec!["name"],
        );
        f(
            r#"SELECT * FROM test where name = \"$name\" and full_name = $name"#,
            r#"SELECT * FROM test where name = \"?1\" and full_name = ?1"#,
            vec!["name"],
        );
    }
}
