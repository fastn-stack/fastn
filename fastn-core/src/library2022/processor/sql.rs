#[derive(Debug)]
pub struct DatabaseConfig {
    pub db_url: String,
    pub db_type: String,
}

impl DatabaseConfig {
    pub fn new(db_url: String, db_type: String) -> DatabaseConfig {
        DatabaseConfig { db_url, db_type }
    }
}

pub(crate) fn get_db_config() -> ftd::interpreter::Result<DatabaseConfig> {
    let db_url = std::env::var("FASTN_DB_URL")
        .map_err(|_| ftd::interpreter::Error::OtherError("FASTN_DB_URL is not set".to_string()))?;

    if let Some(db_url) = db_url.strip_prefix("sqlite:///") {
        return Ok(DatabaseConfig::new(
            db_url.to_string(),
            "sqlite".to_string(),
        ));
    }

    let url = url::Url::parse(&db_url)
        .map_err(|_| ftd::interpreter::Error::OtherError("Invalid DB URL".to_string()))?;

    Ok(DatabaseConfig::new(db_url, url.scheme().to_string()))
}

pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let (headers, query) = super::sqlite::get_p1_data("sql", &value, doc.name)?;

    let db_config = match headers.get_optional_string_by_key("db", doc.name, value.line_number())? {
        Some(url) => match fastn_core::google_sheets::extract_google_sheets_id(url.as_str()) {
            Some(google_sheet_id) => {
                let db_url =
                    fastn_core::google_sheets::generate_google_sheet_url(google_sheet_id.as_str());
                DatabaseConfig::new(db_url, "google_sheets".to_string())
            }
            None => DatabaseConfig::new(url, "sqlite".to_string()),
        },
        None => fastn_core::library2022::processor::sql::get_db_config()?,
    };

    let db_type = db_config.db_type.as_str();

    match db_type {
        "postgres" => Ok(fastn_core::library2022::processor::pg::process(value, kind, doc).await?),
        "sqlite" => Ok(fastn_core::library2022::processor::sqlite::process(
            value,
            kind,
            doc,
            config,
            &db_config,
            headers,
            query.as_str(),
        )
        .await?),
        "google_sheets" => Ok(fastn_core::library2022::processor::google_sheets::process(
            value,
            kind,
            doc,
            &db_config,
            headers,
            query.as_str(),
        )
        .await?),
        _ => unimplemented!("Database currently not supported."),
    }
}

pub const STATUS_OK: usize = 0;
pub const STATUS_ERROR: usize = 1;
const BACKSLASH: char = '\\';
const SPECIAL_CHARS: [char; 9] = [BACKSLASH, '$', '/', ':', '"', ',', '\'', ';', ' '];

// TODO: Can improve the performance
// Maybe I should use RegEx?

pub(crate) fn extract_arguments(query: &str) -> ftd::interpreter::Result<(String, Vec<String>)> {
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
                    output_query += &format!("${}", index + 1);
                } else {
                    args.push(arg.clone());
                    let index = args.len();
                    output_query += &format!("${}", index);
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
        // TODO: THROW SOME ERROR, A QUOTE WAS LEFT OPEN
        println!("Quote Open");
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
    }
}
