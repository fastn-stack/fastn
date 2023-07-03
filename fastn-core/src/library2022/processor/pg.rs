async fn create_pool() -> Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError> {
    let mut cfg = deadpool_postgres::Config::new();
    cfg.libpq_style_connection_string = Some(std::env::var("FASTN_PG_URL").unwrap());
    cfg.manager = Some(deadpool_postgres::ManagerConfig {
        recycling_method: deadpool_postgres::RecyclingMethod::Verified,
    });
    let runtime = Some(deadpool_postgres::Runtime::Tokio1);
    match std::env::var("FASTN_PG_CERTIFICATE") {
        Ok(cert) => {
            let cert = tokio::fs::read(cert).await.unwrap();
            let cert = native_tls::Certificate::from_pem(&cert).unwrap();
            let connector = native_tls::TlsConnector::builder()
                .add_root_certificate(cert)
                .build()
                .unwrap();
            let tls = postgres_native_tls::MakeTlsConnector::new(connector);
            cfg.create_pool(runtime, tls)
        }
        _ => cfg.create_pool(runtime, tokio_postgres::NoTls),
    }
}

// TODO: I am a little confused about the use of `tokio::sync` here, both sides are async, so why
//       do we need to use `tokio::sync`? Am I doing something wrong? How do I prove/verify that
//       this is correct?
static POOL_RESULT: tokio::sync::OnceCell<
    Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError>,
> = tokio::sync::OnceCell::const_new();

async fn pool() -> &'static Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError> {
    POOL_RESULT.get_or_init(create_pool).await
}

pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let (headers, query) = super::sqlite::get_p1_data(&value, doc.name)?;

    super::sqlite::result_to_value(
        execute_query(
            query.as_str(),
            doc.name,
            value.line_number(),
            super::sqlite::get_params(&headers, doc)?,
        )
        .await?,
        kind,
        doc,
        value.line_number(),
    )
}

async fn execute_query(
    query: &str,
    doc_name: &str,
    line_number: usize,
    query_params: Vec<String>,
) -> ftd::interpreter::Result<Vec<Vec<serde_json::Value>>> {
    let client = pool().await.as_ref().unwrap().get().await.unwrap();
    let stmt = client.prepare_cached(query).await.unwrap();
    let count = stmt.columns().len();
    let query_params = query_params
        .iter()
        .map(|value| value as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect::<Vec<_>>();

    let rows = client.query(&stmt, &query_params).await.unwrap();
    let mut result: Vec<Vec<serde_json::Value>> = vec![];

    for r in rows {
        result.push(row_to_json(r, count, doc_name, line_number)?)
    }

    Ok(result)
}

fn row_to_json(
    r: tokio_postgres::Row,
    count: usize,
    doc_name: &str,
    line_number: usize,
) -> ftd::interpreter::Result<Vec<serde_json::Value>> {
    let mut row: Vec<serde_json::Value> = Vec::with_capacity(count);
    for i in 0..count {
        match r.try_get::<usize, _>(i) {
            Ok(value) => row.push(value),
            Err(e) => {
                return ftd::interpreter::utils::e2(
                    format!("Can't get JSON Value at {}: {}", i, e),
                    doc_name,
                    line_number,
                );
            }
        }
    }

    Ok(row)
}
