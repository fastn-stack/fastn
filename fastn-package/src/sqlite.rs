const DB_URL: &str = "file::memory:?cache=shared";

fn rw_flags() -> rusqlite::OpenFlags {
    rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_URI
}

// fn ro_flags() -> rusqlite::OpenFlags {
//     rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI
// }

pub(crate) fn initialize_db(
) -> Result<rusqlite::Connection, fastn_issues::initialization::InitializeDBError> {
    let conn = rusqlite::Connection::open_with_flags(DB_URL, rw_flags()).map_err(|e| {
        fastn_issues::initialization::InitializeDBError::OpenDBConnection { source: e }
    })?;
    conn.execute_batch(include_str!("../create-db.sql"))
        .map_err(|e| fastn_issues::initialization::InitializeDBError::CreateTables { source: e })?;
    Ok(conn)
}
