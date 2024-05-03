pub async fn query(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    conn: i32,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let q: fastn_ds::wasm::exports::pg::Query =
        fastn_ds::wasm::helpers::get_json(ptr, len, &mut caller)?;
    let res = caller.data_mut().pg_query(conn, q).await?;
    fastn_ds::wasm::helpers::send_json(res, &mut caller).await
}

#[derive(serde::Serialize, Debug)]
pub struct Cursor {
    columns: Vec<Column>,
    rows: Vec<PgRow>,
}

#[derive(serde::Serialize, Debug)]
struct Column {
    name: String,
    oid: u32,
}

#[derive(serde::Serialize, Debug)]
struct PgRow {
    fields: Vec<Option<Vec<u8>>>,
}

struct PgField {
    bytes: Option<Vec<u8>>,
}

impl<'a> tokio_postgres::types::FromSql<'a> for PgField {
    fn from_sql(
        _ty: &tokio_postgres::types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(PgField {
            bytes: Some(raw.into()),
        })
    }

    fn from_sql_null(
        _ty: &tokio_postgres::types::Type,
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(PgField { bytes: None })
    }

    fn from_sql_nullable(
        _ty: &tokio_postgres::types::Type,
        raw: Option<&'a [u8]>,
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(PgField {
            bytes: raw.map(|v| v.into()),
        })
    }

    fn accepts(_ty: &tokio_postgres::types::Type) -> bool {
        true
    }
}

impl Column {
    pub fn from_pg(c: &tokio_postgres::Column) -> Self {
        Column {
            name: c.name().to_string(),
            oid: c.type_().oid(),
        }
    }
}

impl Cursor {
    async fn from_stream(
        stream: tokio_postgres::RowStream,
    ) -> Result<Cursor, tokio_postgres::Error> {
        use futures_util::TryStreamExt;

        futures_util::pin_mut!(stream);

        let mut rows = vec![];
        let mut columns: Option<Vec<Column>> = None;

        while let Some(row) = stream.try_next().await? {
            if columns.is_none() {
                columns = Some(row.columns().iter().map(Column::from_pg).collect());
            }

            rows.push(PgRow::from_row(row));
        }

        Ok(Cursor {
            columns: columns.unwrap_or_default(),
            rows,
        })
    }
}

impl PgRow {
    pub fn from_row(row: tokio_postgres::Row) -> Self {
        let mut fields = vec![];
        for i in 0..row.len() {
            let f: PgField = row.get(i);
            fields.push(f.bytes);
        }

        PgRow { fields }
    }
}

impl fastn_ds::wasm::Store {
    pub async fn pg_query(
        &mut self,
        conn: i32,
        q: fastn_ds::wasm::exports::pg::Query,
    ) -> wasmtime::Result<Result<Cursor, ft_sys_shared::DbError>> {
        let mut clients = self.clients.lock().await;
        let client = match clients.get_mut(conn as usize) {
            Some(c) => c,
            None => panic!(
                "unknown connection asked: {conn}, have {} connections",
                clients.len()
            ),
        };

        Ok(
            match client.client.query_raw(q.sql.as_str(), q.binds).await {
                Ok(stream) => Cursor::from_stream(stream)
                    .await
                    .map_err(fastn_ds::wasm::exports::pg::pg_to_shared),
                Err(e) => Err(fastn_ds::wasm::exports::pg::pg_to_shared(e)),
            },
        )
    }
}
