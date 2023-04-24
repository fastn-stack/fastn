const DB_URL: &str = "file::memory:?cache=shared";

#[derive(thiserror::Error, Debug)]
pub enum InitializeDBError {
    #[error("cant open db connection: {source}")]
    CantOpenDBConnection { source: rusqlite::Error },
    #[error("cant create tables: {source}")]
    CantCreateTables { source: rusqlite::Error },
}

fn rw_flags() -> rusqlite::OpenFlags {
    rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_URI
}

// fn ro_flags() -> rusqlite::OpenFlags {
//     rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI
// }

pub(crate) fn initialize_db() -> Result<rusqlite::Connection, InitializeDBError> {
    let conn = rusqlite::Connection::open_with_flags(DB_URL, rw_flags())
        .map_err(|e| InitializeDBError::CantOpenDBConnection { source: e })?;
    conn.execute_batch(include_str!("../create-db.sql"))
        .map_err(|e| InitializeDBError::CantCreateTables { source: e })?;
    Ok(conn)
}
