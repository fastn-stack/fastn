pub async fn query<STORE: fastn_wasm::StoreExt>(
    mut caller: wasmtime::Caller<'_, fastn_wasm::Store<STORE>>,
    _conn: i32,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let q: Query = fastn_wasm::helpers::get_json(ptr, len, &mut caller)?;
    let res = caller.data_mut().sqlite_query(q).await?;
    fastn_wasm::helpers::send_json(res, &mut caller).await
}

#[derive(serde::Deserialize, Debug)]
pub struct Query {
    pub sql: String,
    pub binds: Vec<ft_sys_shared::SqliteRawValue>,
}

#[derive(serde::Serialize, Debug)]
pub struct Cursor {
    columns: Vec<String>,
    rows: Vec<Row>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum SqliteType {
    /// Bind using `sqlite3_bind_blob`
    Binary,
    /// Bind using `sqlite3_bind_text`
    Text,
    /// `bytes` should contain an `f32`
    Float,
    /// `bytes` should contain an `f64`
    Double,
    /// `bytes` should contain an `i16`
    SmallInt,
    /// `bytes` should contain an `i32`
    Integer,
    /// `bytes` should contain an `i64`
    Long,
}

#[derive(serde::Serialize, Debug)]
struct Row {
    fields: Vec<ft_sys_shared::SqliteRawValue>,
}

impl Row {
    fn from_sqlite(len: usize, row: &rusqlite::Row<'_>) -> Self {
        let mut fields = vec![];
        for i in 0..len {
            let field = row.get_ref_unwrap(i);
            let field = match field {
                rusqlite::types::ValueRef::Null => ft_sys_shared::SqliteRawValue::Null,
                rusqlite::types::ValueRef::Integer(i) => ft_sys_shared::SqliteRawValue::Integer(i),
                rusqlite::types::ValueRef::Real(f) => ft_sys_shared::SqliteRawValue::Real(f),
                rusqlite::types::ValueRef::Text(s) => {
                    ft_sys_shared::SqliteRawValue::Text(String::from_utf8_lossy(s).to_string())
                }
                rusqlite::types::ValueRef::Blob(b) => {
                    ft_sys_shared::SqliteRawValue::Blob(b.to_vec())
                }
            };
            fields.push(field);
        }
        Self { fields }
    }
}

#[allow(dead_code)]
struct Field {
    bytes: Option<ft_sys_shared::SqliteRawValue>,
}

impl<STORE: fastn_wasm::StoreExt> fastn_wasm::Store<STORE> {
    pub async fn sqlite_query(
        &mut self,
        q: Query,
    ) -> wasmtime::Result<Result<Cursor, ft_sys_shared::DbError>> {
        let conn = match self.sqlite {
            Some(ref mut conn) => conn,
            None => {
                return Ok(Err(ft_sys_shared::DbError::UnableToSendCommand(
                    "No connection".into(),
                )));
            }
        };

        let conn = conn.lock().await;
        println!("query1: {q:?}");
        let mut stmt = match conn.prepare(q.sql.as_str()) {
            Ok(v) => v,
            Err(fastn_wasm::SQLError::Rusqlite(e)) => {
                eprint!("err: {e:?}");
                let e = rusqlite_to_diesel(e);
                eprintln!("err: {e:?}");
                return Ok(Err(e));
            }
            Err(fastn_wasm::SQLError::InvalidQuery(e)) => {
                return Ok(Err(ft_sys_shared::DbError::UnableToSendCommand(e)));
            } // Todo: Handle error message
        };

        let columns: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let mut rows = vec![];
        let mut r = match stmt.query(rusqlite::params_from_iter(q.binds)) {
            Ok(v) => v,
            Err(e) => {
                eprint!("err: {e:?}");
                let e = rusqlite_to_diesel(e);
                eprintln!("err: {e:?}");
                return Ok(Err(e));
            }
        };

        loop {
            match r.next() {
                Ok(Some(row)) => {
                    rows.push(Row::from_sqlite(columns.len(), row));
                }
                Ok(None) => break,
                Err(e) => {
                    eprint!("err: {e:?}");
                    let e = rusqlite_to_diesel(e);
                    eprintln!("err: {e:?}");
                    return Ok(Err(e));
                }
            }
        }
        println!("found result, {columns:?}, {rows:?}");

        Ok(Ok(Cursor { columns, rows }))
    }
}

pub fn rusqlite_to_diesel(e: rusqlite::Error) -> ft_sys_shared::DbError {
    match e {
        rusqlite::Error::SqliteFailure(
            libsqlite3_sys::Error {
                extended_code,
                code,
            },
            message,
        ) => ft_sys_shared::DbError::DatabaseError {
            kind: code_to_kind(extended_code),
            message: message.unwrap_or_else(|| format!("{code:?}")),
            details: None,
            hint: None,
            table_name: None,
            column_name: None,
            constraint_name: None,
            statement_position: None,
        },
        e => ft_sys_shared::DbError::UnableToSendCommand(e.to_string()),
    }
}

fn code_to_kind(code: std::os::raw::c_int) -> ft_sys_shared::DatabaseErrorKind {
    // borrowed from diesel/sqlite/last_error function
    match code {
        libsqlite3_sys::SQLITE_CONSTRAINT_UNIQUE | libsqlite3_sys::SQLITE_CONSTRAINT_PRIMARYKEY => {
            ft_sys_shared::DatabaseErrorKind::UniqueViolation
        }
        libsqlite3_sys::SQLITE_CONSTRAINT_FOREIGNKEY => {
            ft_sys_shared::DatabaseErrorKind::ForeignKeyViolation
        }
        libsqlite3_sys::SQLITE_CONSTRAINT_NOTNULL => {
            ft_sys_shared::DatabaseErrorKind::NotNullViolation
        }
        libsqlite3_sys::SQLITE_CONSTRAINT_CHECK => ft_sys_shared::DatabaseErrorKind::CheckViolation,
        _ => ft_sys_shared::DatabaseErrorKind::Unknown,
    }
}
