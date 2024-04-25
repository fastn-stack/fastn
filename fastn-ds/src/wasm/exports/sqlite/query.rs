pub async fn query(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    _conn: i32,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let q: Query = fastn_ds::wasm::helpers::get_json(ptr, len, &mut caller).await?;
    let res = caller.data_mut().sqlite_query(q).await?;
    fastn_ds::wasm::helpers::send_json(res, &mut caller).await
}

#[derive(serde::Deserialize, Debug)]
pub struct Query {
    sql: String,
    binds: Vec<Value>,
}

#[derive(serde::Serialize, Debug)]
pub struct Cursor {
    columns: Vec<String>,
    rows: Vec<Row>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl rusqlite::types::ToSql for Value {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput> {
        match self {
            Value::Null => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Null,
            )),
            Value::Integer(i) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(*i),
            )),
            Value::Real(f) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Real(*f),
            )),
            Value::Text(s) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Text(s.clone()),
            )),
            Value::Blob(b) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Blob(b.clone()),
            )),
        }
    }
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
    fields: Vec<Value>,
}

impl Row {
    fn from_sqlite(len: usize, row: &rusqlite::Row<'_>) -> Self {
        let mut fields = vec![];
        for i in 0..len {
            let field = row.get_ref_unwrap(i);
            let field = match field {
                rusqlite::types::ValueRef::Null => Value::Null,
                rusqlite::types::ValueRef::Integer(i) => Value::Integer(i),
                rusqlite::types::ValueRef::Real(f) => Value::Real(f),
                rusqlite::types::ValueRef::Text(s) => {
                    Value::Text(String::from_utf8_lossy(s).to_string())
                }
                rusqlite::types::ValueRef::Blob(b) => Value::Blob(b.to_vec()),
            };
            fields.push(field);
        }
        Self { fields }
    }
}

#[allow(dead_code)]
struct Field {
    bytes: Option<Value>,
}

impl fastn_ds::wasm::Store {
    pub async fn sqlite_query(
        &mut self,
        q: Query,
    ) -> wasmtime::Result<Result<Cursor, ft_sys_shared::DbError>> {
        let conn = self.sqlite.as_ref().lock().await;
        match conn.as_ref() {
            Some(conn) => {
                println!("conn, sql: {}", q.sql.as_str());
                let mut stmt = conn.prepare(q.sql.as_str())?;
                println!("stmt");

                let columns: Vec<String> = stmt
                    .column_names()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect();

                let mut rows = vec![];
                let mut r = stmt.query(rusqlite::params_from_iter(q.binds))?;

                while let Ok(Some(row)) = r.next() {
                    rows.push(Row::from_sqlite(columns.len(), row));
                }

                Ok(Ok(Cursor { columns, rows }))
            }
            None => {
                println!("no conn");
                Ok(Err(ft_sys_shared::DbError::UnableToSendCommand(
                    "no connection".to_string(),
                )))
            }
        }
    }
}
