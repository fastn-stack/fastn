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

/// Mountpoint is the path on which the wasm file is mounted.
///
/// If in FASTN.ftd, we have:
///
/// ```ftd
/// -- import: fastn
/// -- fastn.package: hello-world
/// -- fastn.url-mappings:
/// /foo/* -> wasm+proxy://hello-world.wasm/*
/// ```
///
/// Then the `mountpoint` is `/foo/`.
pub const FASTN_MOUNTPOINT: &str = "x-fastn-mountpoint";

/// A json object that contains the mountpoints of `fastn.app`s
///
/// If in FASTN.ftd, we have:
///
/// ```ftd
/// -- import: fastn
/// -- fastn.package: hello-world
///
/// -- fastn.app: Auth App
/// package: lets-auth.fifthtry.site
/// mount-point: /-/auth/
///
/// -- fastn.app: Let's Talk App
/// package: lets-talk.fifthtry.site
/// mount-point: /talk/
/// ```
///
/// Then the value will be a json string:
///
/// ```json
/// { "lets-auth": "/-/auth/", "lets-talk": "/talk/" }
/// ```
///
/// NOTE: `lets-auth.fifthtry.site` and `lets-talk.fifthtry.site` are required to be a system
/// package. The names `lets-auth` and `lets-talk` are taken from their `system` field
pub const FASTN_APP_MOUNTS: &str = "x-fastn-app-mounts";
