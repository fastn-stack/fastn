pub async fn processor<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    tokio::task::block_in_place(move || processor_(section, doc, config))
}

pub fn processor_(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    let db = match section
        .header
        .str_optional(doc.name, section.line_number, "db")?
    {
        Some(v) => v,
        None => {
            return ftd::e2(
                "`db` is not specified".to_string(),
                doc.name,
                section.line_number,
            )
        }
    };
    let query = section.body(section.line_number, doc.name)?;

    let conn =
        match rusqlite::Connection::open_with_flags(db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        {
            Ok(conn) => conn,
            Err(e) => {
                let original_path = match config.original_path() {
                    Ok(original_path) => original_path,
                    _ => {
                        return ftd::e2(
                            format!("Failed to open `{}`: {:?}", db, e),
                            doc.name,
                            section.line_number,
                        );
                    }
                };
                let db_path = original_path.join(db);
                match rusqlite::Connection::open_with_flags(
                    &db_path,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
                ) {
                    Ok(conn) => conn,
                    Err(e) => {
                        return ftd::e2(
                            format!("Failed to open `{}`: {:?}", db_path.as_str(), e),
                            doc.name,
                            section.line_number,
                        )
                    }
                }
            }
        };

    let mut stmt = match conn.prepare(query.as_str()) {
        Ok(v) => v,
        Err(e) => {
            return ftd::e2(
                format!("Failed to prepare query: {:?}", e),
                doc.name,
                section.line_number,
            )
        }
    };

    let count = stmt.column_count();

    let mut rows = match stmt.query([]) {
        Ok(v) => v,
        Err(e) => {
            return ftd::e2(
                format!("Failed to prepare query: {:?}", e),
                doc.name,
                section.line_number,
            )
        }
    };

    if is_list(section, doc) {
        let mut result: Vec<Vec<serde_json::Value>> = vec![];
        loop {
            match rows.next() {
                Ok(None) => break,
                Ok(Some(r)) => {
                    result.push(row_to_json(r, section, doc, count)?);
                }
                Err(e) => {
                    return ftd::e2(
                        format!("Failed to execute query: {:?}", e),
                        doc.name,
                        section.line_number,
                    )
                }
            }
        }
        doc.from_json_rows(section, &result)
    } else {
        let json = match rows.next() {
            Ok(Some(r)) => row_to_json(r, section, doc, count)?,
            Ok(None) => {
                return ftd::e2(
                    "Query returned no result, expected one row".to_string(),
                    doc.name,
                    section.line_number,
                )
            }
            Err(e) => {
                return ftd::e2(
                    format!("Failed to execute query: {:?}", e),
                    doc.name,
                    section.line_number,
                )
            }
        };
        doc.from_json_row(section, &json)
    }
}

fn row_to_json(
    r: &rusqlite::Row,
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    count: usize,
) -> ftd::p1::Result<Vec<serde_json::Value>> {
    let mut row: Vec<serde_json::Value> = vec![];
    for i in 0..count {
        match r.get::<usize, rusqlite::types::Value>(i) {
            Ok(rusqlite::types::Value::Null) => row.push(serde_json::Value::Null),
            Ok(rusqlite::types::Value::Integer(i)) => row.push(serde_json::Value::Number(i.into())),
            Ok(rusqlite::types::Value::Real(i)) => row.push(serde_json::Value::Number(
                serde_json::Number::from_f64(i).unwrap(),
            )),
            Ok(rusqlite::types::Value::Text(i)) => row.push(serde_json::Value::String(i)),
            Ok(rusqlite::types::Value::Blob(_)) => {
                return ftd::e2(
                    format!("Query returned blob for column: {}", i),
                    doc.name,
                    section.line_number,
                );
            }
            Err(e) => {
                return ftd::e2(
                    format!("Failed to read response: {:?}", e),
                    doc.name,
                    section.line_number,
                );
            }
        }
    }
    Ok(row)
}

fn is_list(section: &ftd::p1::Section, doc: &ftd::p2::TDoc) -> bool {
    matches!(
        doc.get_value(section.line_number, section.name.as_str()),
        Ok(ftd::Value::List { .. })
    )
}
