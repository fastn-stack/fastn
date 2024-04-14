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
    columns: Vec<Column>,
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
struct Column {
    name: String,
    type_: SqliteType,
}

#[derive(serde::Serialize, Debug)]
struct Row {
    fields: Vec<Option<Value>>,
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
        let conn = match self.sqlite {
            Some(ref mut conn) => conn,
            None => {
                return Ok(Err(ft_sys_shared::DbError::UnableToSendCommand(
                    "No connection".into(),
                )))
            }
        };

        let conn = conn.lock().await;
        println!("conn, sql: {}", q.sql.as_str());
        let mut stmt = conn.prepare(q.sql.as_str())?;
        println!("stmt");
        let _rows = stmt.query(rusqlite::params_from_iter(q.binds))?;
        println!("rows");
        todo!()
    }
}
