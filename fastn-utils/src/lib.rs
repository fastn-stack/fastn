#![warn(unused_extern_crates)]
#![deny(unused_crate_dependencies)]

pub mod sql;

#[derive(thiserror::Error, Debug)]
pub enum SqlError {
    #[error("connection error {0}")]
    Connection(rusqlite::Error),
    #[error("Query error {0}")]
    Query(rusqlite::Error),
    #[error("Execute error {0}")]
    Execute(rusqlite::Error),
    #[error("column error {0}: {0}")]
    Column(usize, rusqlite::Error),
    #[error("row error {0}")]
    Row(rusqlite::Error),
    #[error("found blob")]
    FoundBlob,
    #[error("unknown db error")]
    UnknownDB,
}

pub fn rows_to_json(
    mut rows: rusqlite::Rows,
    count: usize,
) -> Result<Vec<Vec<serde_json::Value>>, SqlError> {
    let mut result: Vec<Vec<serde_json::Value>> = vec![];
    loop {
        match rows.next() {
            Ok(None) => break,
            Ok(Some(r)) => {
                result.push(row_to_json(r, count)?);
            }
            Err(e) => return Err(SqlError::Row(e)),
        }
    }
    Ok(result)
}

pub fn row_to_json(r: &rusqlite::Row, count: usize) -> Result<Vec<serde_json::Value>, SqlError> {
    let mut row: Vec<serde_json::Value> = Vec::with_capacity(count);
    for i in 0..count {
        match r.get::<usize, rusqlite::types::Value>(i) {
            Ok(rusqlite::types::Value::Null) => row.push(serde_json::Value::Null),
            Ok(rusqlite::types::Value::Integer(i)) => row.push(serde_json::Value::Number(i.into())),
            Ok(rusqlite::types::Value::Real(i)) => row.push(serde_json::Value::Number(
                serde_json::Number::from_f64(i).unwrap(),
            )),
            Ok(rusqlite::types::Value::Text(i)) => row.push(serde_json::Value::String(i)),
            Ok(rusqlite::types::Value::Blob(_)) => return Err(SqlError::FoundBlob),
            Err(e) => return Err(SqlError::Column(i, e)),
        }
    }
    Ok(row)
}

pub const FASTN_MOUNTPOINT: &str = "x-fastn-mountpoint";
