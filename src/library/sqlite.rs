pub fn processor(section: &ftd::p1::Section, doc: &ftd::p2::TDoc) -> ftd::p1::Result<ftd::Value> {
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
                return ftd::e2(
                    format!("Failed to open `{}`: {:?}", db, e),
                    doc.name,
                    section.line_number,
                )
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
        from_json_rows(section, doc, &result)
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
        from_json_row(section, doc, &json)
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
        match r.get(i) {
            Ok(o) => row.push(o),
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
    ftd::Variable::list_from_p1(section, doc).is_ok()
}

fn from_json_rows(
    _section: &ftd::p1::Section,
    _doc: &ftd::p2::TDoc,
    _rows: &[Vec<serde_json::Value>],
) -> ftd::p1::Result<ftd::Value> {
    todo!()
}

fn from_json_row(
    _section: &ftd::p1::Section,
    _doc: &ftd::p2::TDoc,
    _row: &[serde_json::Value],
) -> ftd::p1::Result<ftd::Value> {
    todo!()
}
